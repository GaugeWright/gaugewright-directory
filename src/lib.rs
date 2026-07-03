//! The blind directory service (`ACCT-2`, [ADR 0054]) — the readable routing/identity layer
//! of the account model, hosted on the always-on relay (`infra/relay/`) so your identity,
//! devices, and placement pointers are reachable **even with your machine off**.
//!
//! It is **blind**. Per account root it holds:
//! - a [`DirectoryRecord`] — readable routing (root pubkey, device pubkeys, placement
//!   pointers; **no secrets**, `INV-10`), and
//! - an **opaque** sealed account blob — the hex ciphertext [`gaugewright_app::account::seal_account_blob`]
//!   produces. Only your devices' key opens it; the directory holds no key and never calls
//!   [`gaugewright_app::account::open_account_blob`].
//!
//! Reads are public (routing is meant to be found). **Writes are signed and fail-closed**:
//! only the holder of the root key may publish or overwrite its own record — the signature is
//! verified against the record's *own* `root_pubkey` ([`verify_signature`]), so a public
//! directory cannot be hijacked by a stranger overwriting your routing.
//!
//! Durable, append-only backing (`INV-6`): each signed publish appends a [`DirectoryEntry`]
//! record to the reserved `directory` scope, and [`BlindDirectory::open`] rehydrates the
//! in-memory read cache by folding that log **latest-publish-per-root wins** (`INV-5`), so the
//! directory survives a relay restart. [`BlindDirectory::new`] keeps the pure in-memory cache
//! (dev/test). Eventual consistency **across directory replicas** remains the [ADR 0054]
//! follow-on.
//!
//! [ADR 0054]: ../../specs/decisions/0054-account-directory-and-sealed-sync.md

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::put,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use gaugewright_core::ids::PublicKey;
use gaugewright_core::signature::{verify_signature, Signature};
use gaugewright_store::Store;

use gaugewright_app::account::DirectoryRecord;

/// The reserved scope the durable directory log lives in.
pub const DIRECTORY_SCOPE: &str = "directory";
/// The record kind each signed publish appends (one per publish; latest-per-root wins).
pub const DIRECTORY_ENTRY_KIND: &str = "directory_entry";

/// What the blind directory holds for one account root: the readable routing record + the
/// opaque sealed account blob (hex ciphertext the directory cannot open).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub directory: DirectoryRecord,
    /// Opaque hex ciphertext (`INV-10`) — the sealed `AccountBlob`. Never opened here.
    pub sealed_blob: String,
}

/// The canonical bytes a root signs to authorize publishing `entry` (the same function feeds
/// both the signer and [`verify_signature`], so they always agree).
pub fn signing_bytes(entry: &DirectoryEntry) -> Vec<u8> {
    serde_json::to_vec(entry).unwrap_or_default()
}

/// A signed publish request: the entry + the root key's signature over [`signing_bytes`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedDirectoryPut {
    pub entry: DirectoryEntry,
    pub signature: Signature,
}

/// The blind directory store — keyed by account root pubkey. An in-memory read cache over an
/// **optional durable, append-only log** (`store`): `new` is pure in-memory (dev/test); `open`
/// is durable + rehydrated. `Clone` shares both (the router takes it by value).
#[derive(Clone, Default)]
pub struct BlindDirectory {
    inner: Arc<Mutex<HashMap<String, DirectoryEntry>>>,
    /// The durable backing (`INV-6`). `None` ⇒ pure in-memory. Behind `Arc<Mutex<…>>` because
    /// `append_record` needs `&mut`, and clones must share one writer (`INV-7`).
    store: Option<Arc<Mutex<Store>>>,
}

impl BlindDirectory {
    /// A pure in-memory directory (dev/test) — no durability across a restart.
    pub fn new() -> Self {
        Self::default()
    }

    /// A **durable** directory over `store`, rehydrating the read cache from the append-only
    /// log (each `directory_entry` record folded latest-wins per root, `INV-5`). The relay host
    /// gives it a file-backed store so an account's routing survives the relay restarting.
    pub fn open(store: Store) -> Self {
        let mut cache: HashMap<String, DirectoryEntry> = HashMap::new();
        if let Ok(rows) = store.records(DIRECTORY_SCOPE, DIRECTORY_ENTRY_KIND) {
            // In-order fold: a later publish for the same root overwrites the earlier (INV-5).
            for row in rows {
                if let Ok(entry) = serde_json::from_str::<DirectoryEntry>(&row) {
                    cache.insert(entry.directory.root_pubkey.clone(), entry);
                }
            }
        }
        Self {
            inner: Arc::new(Mutex::new(cache)),
            store: Some(Arc::new(Mutex::new(store))),
        }
    }

    pub fn get(&self, root: &str) -> Option<DirectoryEntry> {
        self.inner.lock().unwrap().get(root).cloned()
    }

    fn append_entry(&self, entry: &DirectoryEntry) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let Ok(payload) = serde_json::to_string(entry) else {
            return;
        };
        let _ =
            store
                .lock()
                .unwrap()
                .append_record(DIRECTORY_SCOPE, DIRECTORY_ENTRY_KIND, &payload);
    }

    pub fn put(&self, root: &str, entry: DirectoryEntry) {
        // Durable append first (`INV-6`: the log is the record), then refresh the read cache.
        // A store error never drops the cache update — availability is preserved either way.
        self.append_entry(&entry);
        self.inner.lock().unwrap().insert(root.to_string(), entry);
    }
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The directory HTTP surface: `PUT /directory/:root` (signed publish) + `GET /directory/:root`
/// (public read).
pub fn directory_router(dir: BlindDirectory) -> Router {
    Router::new()
        .route("/directory/:root", put(publish).get(fetch))
        .with_state(dir)
}

/// Publish/overwrite the record for `root` — **fail-closed**: the path must match the record's
/// own `root_pubkey`, and the signature must verify under that key. Only you can write your
/// record.
async fn publish(
    State(dir): State<BlindDirectory>,
    Path(root): Path<String>,
    Json(req): Json<SignedDirectoryPut>,
) -> StatusCode {
    // The claimed identity must be self-consistent (path == the record's own root pubkey).
    if req.entry.directory.root_pubkey != root {
        return StatusCode::BAD_REQUEST;
    }
    // Only the holder of the root key may publish under it.
    let pubkey = PublicKey::new(root.clone());
    match verify_signature(&signing_bytes(&req.entry), &req.signature, &pubkey) {
        Ok(true) => {
            dir.put(&root, req.entry);
            StatusCode::NO_CONTENT
        }
        // Unparseable key or a non-verifying signature → fail closed.
        _ => StatusCode::UNAUTHORIZED,
    }
}

/// Fetch the readable record + sealed blob for `root` (public read; the blob stays sealed).
async fn fetch(
    State(dir): State<BlindDirectory>,
    Path(root): Path<String>,
) -> Result<Json<DirectoryEntry>, StatusCode> {
    dir.get(&root).map(Json).ok_or(StatusCode::NOT_FOUND)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gaugewright_core::signature::SigningKey;

    fn entry_for(root_pubkey: &str) -> DirectoryEntry {
        DirectoryEntry {
            directory: DirectoryRecord {
                root_pubkey: root_pubkey.to_string(),
                device_pubkeys: vec!["dev-1".into(), "dev-2".into()],
                placement_pointers: vec!["relay://20.57.147.64:7900/p1".into()],
            },
            // An opaque sealed blob (shape only — the directory never inspects it).
            sealed_blob: "deadbeefcafe".into(),
        }
    }

    fn signed(sk: &SigningKey, entry: &DirectoryEntry) -> SignedDirectoryPut {
        SignedDirectoryPut {
            entry: entry.clone(),
            signature: sk.sign(&signing_bytes(entry)),
        }
    }

    #[tokio::test]
    async fn signed_publish_then_fetch_round_trips_opaquely() {
        let sk = SigningKey::from_seed(&[3u8; 32]).unwrap();
        let root = sk.public_key().as_str().to_string();
        let dir = BlindDirectory::new();
        let entry = entry_for(&root);

        let code = publish(
            State(dir.clone()),
            Path(root.clone()),
            Json(signed(&sk, &entry)),
        )
        .await;
        assert_eq!(code, StatusCode::NO_CONTENT);

        let got = fetch(State(dir.clone()), Path(root.clone()))
            .await
            .unwrap()
            .0;
        // Round-trips byte-identically — incl. the opaque sealed blob the directory never opened.
        assert_eq!(got, entry);
        assert_eq!(got.sealed_blob, "deadbeefcafe");
    }

    #[tokio::test]
    async fn an_unsigned_or_wrongly_signed_write_is_refused() {
        let sk = SigningKey::from_seed(&[3u8; 32]).unwrap();
        let attacker = SigningKey::from_seed(&[9u8; 32]).unwrap();
        let root = sk.public_key().as_str().to_string();
        let dir = BlindDirectory::new();
        let entry = entry_for(&root);

        // A signature by a DIFFERENT key over the same entry must not authorize the write.
        let forged = SignedDirectoryPut {
            entry: entry.clone(),
            signature: attacker.sign(&signing_bytes(&entry)),
        };
        let code = publish(State(dir.clone()), Path(root.clone()), Json(forged)).await;
        assert_eq!(code, StatusCode::UNAUTHORIZED);
        // Nothing was stored — the hijack failed closed.
        assert!(dir.is_empty());
    }

    #[tokio::test]
    async fn a_path_that_disagrees_with_the_record_is_rejected() {
        let sk = SigningKey::from_seed(&[3u8; 32]).unwrap();
        let root = sk.public_key().as_str().to_string();
        let dir = BlindDirectory::new();
        let entry = entry_for(&root);
        // Publish under a different path than the record's own root_pubkey.
        let code = publish(
            State(dir.clone()),
            Path("some-other-root".to_string()),
            Json(signed(&sk, &entry)),
        )
        .await;
        assert_eq!(code, StatusCode::BAD_REQUEST);
        assert!(dir.is_empty());
    }

    #[tokio::test]
    async fn fetching_an_unknown_root_is_not_found() {
        let dir = BlindDirectory::new();
        let got = fetch(State(dir), Path("nobody".to_string())).await;
        assert!(matches!(got, Err(StatusCode::NOT_FOUND)));
    }

    #[tokio::test]
    async fn a_published_record_survives_a_relay_restart() {
        // ACCT-2 durability: a signed publish lands in the append-only log, so a fresh
        // directory opened over the SAME store (a relay restart) rehydrates the routing —
        // your identity is reachable with your machine off, even across the relay bouncing.
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("directory.db");
        let db_path = db.to_str().unwrap();

        let sk = SigningKey::from_seed(&[3u8; 32]).unwrap();
        let root = sk.public_key().as_str().to_string();
        let entry = entry_for(&root);

        // Publish through a durable directory, then drop it (simulating the relay going down).
        {
            let dir = BlindDirectory::open(Store::open(db_path).unwrap());
            let code = publish(
                State(dir.clone()),
                Path(root.clone()),
                Json(signed(&sk, &entry)),
            )
            .await;
            assert_eq!(code, StatusCode::NO_CONTENT);
        }

        // Reopen over the same on-disk log: the read cache rehydrates from the durable record.
        let restarted = BlindDirectory::open(Store::open(db_path).unwrap());
        assert_eq!(
            restarted.len(),
            1,
            "the published routing survived the restart"
        );
        let got = restarted.get(&root).expect("rehydrated entry");
        assert_eq!(got, entry);
        // The sealed blob is carried opaquely through persistence too (never opened).
        assert_eq!(got.sealed_blob, "deadbeefcafe");
    }

    #[tokio::test]
    async fn a_later_publish_overwrites_the_earlier_on_rehydrate() {
        // INV-5 latest-wins: two publishes for one root fold to the most recent on reopen.
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("directory.db");
        let db_path = db.to_str().unwrap();
        let sk = SigningKey::from_seed(&[7u8; 32]).unwrap();
        let root = sk.public_key().as_str().to_string();

        let mut first = entry_for(&root);
        first.sealed_blob = "0001".into();
        let mut second = entry_for(&root);
        second.sealed_blob = "0002".into();

        {
            let dir = BlindDirectory::open(Store::open(db_path).unwrap());
            assert_eq!(
                publish(
                    State(dir.clone()),
                    Path(root.clone()),
                    Json(signed(&sk, &first))
                )
                .await,
                StatusCode::NO_CONTENT
            );
            assert_eq!(
                publish(
                    State(dir.clone()),
                    Path(root.clone()),
                    Json(signed(&sk, &second))
                )
                .await,
                StatusCode::NO_CONTENT
            );
        }

        let restarted = BlindDirectory::open(Store::open(db_path).unwrap());
        assert_eq!(restarted.len(), 1, "one root, folded latest-wins");
        assert_eq!(restarted.get(&root).unwrap().sealed_blob, "0002");
    }
}
