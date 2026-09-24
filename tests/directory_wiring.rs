use std::fs::File;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use gaugedesk_app::account::DirectoryRecord;
use gaugedesk_app::directory_sync::{
    fetch as production_fetch, publish as production_publish, signed_retract as production_retract,
    signing_bytes as production_signing_bytes, DirectoryEntry as ClientDirectoryEntry,
    SignedDirectoryPut as ClientSignedDirectoryPut,
};
use gaugedesk_app::net_http::HttpClient;
use gaugedesk_core::signature::SigningKey;
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};

struct DirectoryProcess {
    child: Child,
    origin: String,
}

/// What the binary wrote before it stopped, for a failure message.
///
/// The fixture used to send both of its streams to `Stdio::null()`, so a
/// server that panicked, could not open its database, or refused its address
/// failed this test as "did not become ready on loopback" — a statement that
/// ten seconds passed and nothing else. Its own first line says which of
/// those it was.
fn said(log: &Path) -> String {
    match std::fs::read_to_string(log) {
        Ok(text) if !text.trim().is_empty() => format!("; it said: {}", text.trim()),
        Ok(_) => "; it wrote nothing before stopping".to_string(),
        Err(error) => format!("; its log at {} could not be read: {error}", log.display()),
    }
}

impl DirectoryProcess {
    fn start(root: &Path) -> Self {
        let probe = TcpListener::bind("127.0.0.1:0").expect("reserve loopback port");
        let port = probe.local_addr().expect("read loopback address").port();
        drop(probe);

        let ready = root.join("ready");
        let database = root.join("directory.db");
        let _ = std::fs::remove_file(&ready);
        // Both streams to a file rather than to null: kept out of a passing
        // run's output, and read back into the message when the fixture fails.
        let log = root.join("directory.log");
        let sink = File::create(&log).expect("create the directory binary's log");
        let errors = sink.try_clone().expect("share the log with stderr");
        let mut child = Command::new(env!("CARGO_BIN_EXE_gaugewright-directory"))
            .env("GAUGEWRIGHT_DIRECTORY_ADDR", format!("127.0.0.1:{port}"))
            .env("GAUGEWRIGHT_DIRECTORY_READY", &ready)
            .env("GAUGEWRIGHT_DIRECTORY_DB", &database)
            .stdout(Stdio::from(sink))
            .stderr(Stdio::from(errors))
            .spawn()
            .expect("start production directory binary");

        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() {
            // A binary that has already exited is not slow, and waiting the
            // rest of the ten seconds to call it slow loses the distinction.
            if let Some(status) = child.try_wait().expect("poll the directory binary") {
                panic!(
                    "directory exited during startup with {status}{}",
                    said(&log)
                );
            }
            assert!(
                Instant::now() < deadline,
                "directory did not become ready on loopback within 10s{}",
                said(&log)
            );
            thread::sleep(Duration::from_millis(20));
        }

        Self {
            child,
            origin: format!("http://127.0.0.1:{port}"),
        }
    }
}

impl Drop for DirectoryProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn entry(root: &str, sealed_blob: String) -> ClientDirectoryEntry {
    ClientDirectoryEntry {
        generation: 1,
        directory: DirectoryRecord {
            root_pubkey: root.to_string(),
            device_pubkeys: vec!["device-laptop".into(), "device-phone".into()],
            placement_pointers: vec!["relay+wss://relay.invalid/placement".into()],
            home_routes: vec![],
        },
        sealed_blob,
        retracted: false,
    }
}

fn opaque_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn signed(signing_key: &SigningKey, entry: ClientDirectoryEntry) -> ClientSignedDirectoryPut {
    ClientSignedDirectoryPut {
        signature: signing_key
            .sign(&production_signing_bytes(&entry).expect("an entry serializes")),
        entry,
    }
}

fn raw_put(origin: &str, root: &str, body: &ClientSignedDirectoryPut) -> u16 {
    let url = format!("{origin}/directory/{root}");
    match ureq::put(&url).send_json(serde_json::to_value(body).expect("serialize put")) {
        Ok(response) => response.status(),
        Err(ureq::Error::Status(status, _)) => status,
        Err(error) => panic!("directory transport failed: {error}"),
    }
}

#[test]
fn production_client_round_trip() {
    let state = tempfile::tempdir().expect("disposable directory state");
    let process = DirectoryProcess::start(state.path());
    let http = HttpClient::with_timeout(Duration::from_secs(2));

    let owner = SigningKey::from_seed(&[17; 32]).expect("owner key");
    let attacker = SigningKey::from_seed(&[23; 32]).expect("attacker key");
    let root = owner.public_key().as_str().to_string();
    let original = entry(&root, "opaque-ciphertext-not-plaintext".into());
    let admitted = signed(&owner, original.clone());

    assert!(
        production_fetch(&http, &process.origin, &root)
            .expect("production client fetch")
            .is_none(),
        "an unknown account root is absent"
    );
    production_publish(&http, &process.origin, &admitted)
        .expect("production client signed publish");
    assert_eq!(
        production_fetch(&http, &process.origin, &root)
            .expect("production client readback")
            .map(|record| record.entry),
        Some(original.clone()),
        "the pinned production GaugeDesk client reads back the exact admitted record"
    );

    drop(process);
    let process = DirectoryProcess::start(state.path());
    assert_eq!(
        production_fetch(&http, &process.origin, &root)
            .expect("readback after process restart")
            .map(|record| record.entry),
        Some(original.clone()),
        "the admitted record survives a production-binary restart"
    );

    let mut forged_entry = original.clone();
    forged_entry.sealed_blob = "attacker-replacement".into();
    let forged = signed(&attacker, forged_entry);
    assert_eq!(raw_put(&process.origin, &root, &forged), 401);
    assert_eq!(
        production_fetch(&http, &process.origin, &root)
            .expect("read after denied forgery")
            .map(|record| record.entry),
        Some(original.clone()),
        "a denied wrong-key mutation leaves authoritative state unchanged"
    );

    let mismatched_path = attacker.public_key().as_str().to_string();
    assert_eq!(raw_put(&process.origin, &mismatched_path, &admitted), 422);
    assert_eq!(
        production_fetch(&http, &process.origin, &root)
            .expect("read after path mismatch")
            .map(|record| record.entry),
        Some(original),
        "a path/root mismatch leaves authoritative state unchanged"
    );

    let malformed_url = format!("{}/directory/{root}", process.origin);
    let malformed = match ureq::put(&malformed_url)
        .set("Content-Type", "application/json")
        .send_string("{")
    {
        Ok(response) => response.status(),
        Err(ureq::Error::Status(status, _)) => status,
        Err(error) => panic!("directory transport failed: {error}"),
    };
    assert_eq!(malformed, 422);
}

#[test]
fn generated_signed_records_round_trip_and_forgery_never_mutates() {
    let state = tempfile::tempdir().expect("disposable directory state");
    let process = DirectoryProcess::start(state.path());
    let http = HttpClient::with_timeout(Duration::from_secs(2));
    let mut runner = TestRunner::new(Config {
        cases: 48,
        failure_persistence: None,
        ..Config::default()
    });
    let strategy = (
        any::<[u8; 32]>(),
        proptest::collection::vec(any::<u8>(), 0..1024),
    );

    runner
        .run(&strategy, |(seed, opaque_bytes)| {
            let owner = SigningKey::from_seed(&seed).expect("generated owner key");
            let root = owner.public_key().as_str().to_string();
            let original = entry(&root, opaque_hex(&opaque_bytes));
            let admitted = signed(&owner, original.clone());
            prop_assert!(
                production_publish(&http, &process.origin, &admitted).is_ok(),
                "valid production-client publish must succeed"
            );
            prop_assert_eq!(
                production_fetch(&http, &process.origin, &root)
                    .expect("generated readback")
                    .map(|record| record.entry),
                Some(original.clone())
            );

            let mut attacker_seed = seed;
            attacker_seed[0] ^= 0xff;
            if attacker_seed == seed {
                attacker_seed[1] ^= 0x01;
            }
            let attacker = SigningKey::from_seed(&attacker_seed).expect("generated attacker key");
            let mut forged_entry = original.clone();
            forged_entry.sealed_blob.push_str("00");
            let forged = signed(&attacker, forged_entry);
            prop_assert_eq!(raw_put(&process.origin, &root, &forged), 401);
            prop_assert_eq!(
                production_fetch(&http, &process.origin, &root)
                    .expect("generated post-denial readback")
                    .map(|record| record.entry),
                Some(original)
            );
            Ok(())
        })
        .expect("generated real-transport directory contract");
}

#[test]
fn production_client_retracts_and_the_fence_holds_across_it() {
    let state = tempfile::tempdir().expect("disposable directory state");
    let process = DirectoryProcess::start(state.path());
    let http = HttpClient::with_timeout(Duration::from_secs(2));

    let owner = SigningKey::from_seed(&[31; 32]).expect("owner key");
    let root = owner.public_key().as_str().to_string();
    let first = entry(&root, "0001".into());
    production_publish(&http, &process.origin, &signed(&owner, first.clone()))
        .expect("first publish");

    // The pinned production client's retraction advances the generation like any publish,
    // and the root then reads as Gone, with the empty retraction as the body.
    let retraction = production_retract(&owner, 2).expect("a retraction at generation two");
    production_publish(&http, &process.origin, &retraction).expect("retraction publish");
    let read = production_fetch(&http, &process.origin, &root)
        .expect("the client reads a 410 as the retraction")
        .expect("a retracted root still returns its retraction");
    assert!(read.entry.retracted);
    assert_eq!(read.entry.generation, 2);
    assert_eq!(
        public_status(&process.origin, &root),
        410,
        "a public reader sees the root as Gone"
    );

    // An old publish cannot be replayed to un-retract, and the owner re-appears by
    // advancing past the retraction.
    let mut replay = first.clone();
    replay.sealed_blob = "beef".into();
    assert_eq!(
        raw_put(&process.origin, &root, &signed(&owner, replay)),
        409
    );
    let mut again = entry(&root, "0003".into());
    again.generation = 3;
    production_publish(&http, &process.origin, &signed(&owner, again.clone()))
        .expect("re-publish after retraction");
    assert_eq!(
        production_fetch(&http, &process.origin, &root)
            .expect("readback")
            .map(|record| record.entry),
        Some(again)
    );
}

fn public_status(origin: &str, root: &str) -> u16 {
    match ureq::get(&format!("{origin}/directory/{root}")).call() {
        Ok(response) => response.status(),
        Err(ureq::Error::Status(status, _)) => status,
        Err(error) => panic!("directory transport failed: {error}"),
    }
}
