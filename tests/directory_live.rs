//! Live blind-directory verification (`ACCT-2`, [ADR 0054]) — drives the **deployed**
//! directory service over real HTTP and confirms the signed-publish / public-fetch contract
//! end-to-end (the unit tests in `directory.rs` cover the handler logic; this closes the loop
//! through the wire + axum serialization against the running service).
//!
//! `#[ignore]`d: needs `GAUGEWRIGHT_DIRECTORY_URL` (e.g. `http://20.57.147.64:7901`). Run via
//! `scripts/directory-check.sh`.

use gaugewright_app::account::DirectoryRecord;
use gaugewright_core::signature::SigningKey;
use gaugewright_directory::{signing_bytes, DirectoryEntry, SignedDirectoryPut};

fn env_or_skip(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => Some(v.trim_end_matches('/').to_string()),
        _ => {
            eprintln!("SKIP directory_live: ${key} unset (run via scripts/directory-check.sh)");
            None
        }
    }
}

#[test]
#[ignore = "needs GAUGEWRIGHT_DIRECTORY_URL (ACCT-2)"]
fn signed_publish_and_public_fetch_against_the_live_directory() {
    let Some(base) = env_or_skip("GAUGEWRIGHT_DIRECTORY_URL") else {
        return;
    };

    let sk = SigningKey::from_seed(&[5u8; 32]).expect("valid seed");
    let root = sk.public_key().as_str().to_string();
    let entry = DirectoryEntry {
        directory: DirectoryRecord {
            root_pubkey: root.clone(),
            device_pubkeys: vec!["dev-laptop".into(), "dev-phone".into()],
            placement_pointers: vec!["relay://20.57.147.64:7900/p1".into()],
        },
        sealed_blob: "0a1b2c3d4e5f".into(), // opaque to the directory
    };
    let put = SignedDirectoryPut {
        entry: entry.clone(),
        signature: sk.sign(&signing_bytes(&entry)),
    };

    let url = format!("{base}/directory/{root}");

    // Signed publish → 204.
    let (code, _) = http_put_json(&url, &serde_json::to_value(&put).unwrap());
    assert_eq!(code, 204, "a validly-signed publish should be accepted");

    // Public fetch → the readable record + opaque blob, byte-identical.
    let got: DirectoryEntry = ureq::get(&url)
        .call()
        .expect("fetch should succeed")
        .into_json()
        .expect("decode entry");
    assert_eq!(
        got, entry,
        "the directory round-trips the record + sealed blob"
    );

    // A write signed by the WRONG key over the same root must be refused (no hijack).
    let attacker = SigningKey::from_seed(&[9u8; 32]).expect("valid seed");
    let forged = SignedDirectoryPut {
        entry: entry.clone(),
        signature: attacker.sign(&signing_bytes(&entry)),
    };
    let (code, _) = http_put_json(&url, &serde_json::to_value(&forged).unwrap());
    assert_eq!(
        code, 401,
        "a write not signed by the root key must fail closed"
    );

    println!("directory_live: signed publish 204, fetch round-trips, forged write 401 ✔");
}

/// PUT JSON, returning `(status, body)` (a non-2xx is a status, not an error).
fn http_put_json(url: &str, body: &serde_json::Value) -> (u16, String) {
    match ureq::put(url).send_json(body.clone()) {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}
