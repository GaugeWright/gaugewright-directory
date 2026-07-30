use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use gaugewright_app::account::DirectoryRecord;
use gaugewright_app::directory_sync::{
    fetch as production_fetch, publish as production_publish,
    signing_bytes as production_signing_bytes, DirectoryEntry as ClientDirectoryEntry,
    SignedDirectoryPut as ClientSignedDirectoryPut,
};
use gaugewright_app::net_http::HttpClient;
use gaugewright_core::signature::SigningKey;
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};

struct DirectoryProcess {
    child: Child,
    origin: String,
}

impl DirectoryProcess {
    fn start(root: &Path) -> Self {
        let probe = TcpListener::bind("127.0.0.1:0").expect("reserve loopback port");
        let port = probe.local_addr().expect("read loopback address").port();
        drop(probe);

        let ready = root.join("ready");
        let database = root.join("directory.db");
        let _ = std::fs::remove_file(&ready);
        let child = Command::new(env!("CARGO_BIN_EXE_gaugewright-directory"))
            .env("GAUGEWRIGHT_DIRECTORY_ADDR", format!("127.0.0.1:{port}"))
            .env("GAUGEWRIGHT_DIRECTORY_READY", &ready)
            .env("GAUGEWRIGHT_DIRECTORY_DB", &database)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start production directory binary");

        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() {
            assert!(
                Instant::now() < deadline,
                "directory did not become ready on loopback"
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
        directory: DirectoryRecord {
            root_pubkey: root.to_string(),
            device_pubkeys: vec!["device-laptop".into(), "device-phone".into()],
            placement_pointers: vec!["relay+wss://relay.invalid/placement".into()],
        },
        sealed_blob,
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
        signature: signing_key.sign(&production_signing_bytes(&entry)),
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
        production_fetch(&http, &process.origin, &root).expect("production client readback"),
        Some(original.clone()),
        "the pinned production GaugeDesk client reads back the exact admitted record"
    );

    drop(process);
    let process = DirectoryProcess::start(state.path());
    assert_eq!(
        production_fetch(&http, &process.origin, &root).expect("readback after process restart"),
        Some(original.clone()),
        "the admitted record survives a production-binary restart"
    );

    let mut forged_entry = original.clone();
    forged_entry.sealed_blob = "attacker-replacement".into();
    let forged = signed(&attacker, forged_entry);
    assert_eq!(raw_put(&process.origin, &root, &forged), 401);
    assert_eq!(
        production_fetch(&http, &process.origin, &root).expect("read after denied forgery"),
        Some(original.clone()),
        "a denied wrong-key mutation leaves authoritative state unchanged"
    );

    let mismatched_path = attacker.public_key().as_str().to_string();
    assert_eq!(raw_put(&process.origin, &mismatched_path, &admitted), 400);
    assert_eq!(
        production_fetch(&http, &process.origin, &root).expect("read after path mismatch"),
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
    assert_eq!(malformed, 400);
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
                production_fetch(&http, &process.origin, &root).expect("generated readback"),
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
                    .expect("generated post-denial readback"),
                Some(original)
            );
            Ok(())
        })
        .expect("generated real-transport directory contract");
}
