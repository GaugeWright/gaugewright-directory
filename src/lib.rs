//! The blind directory service (`ACCT-2`, [ADR 0054]) — the readable routing/identity layer
//! of the account model, so your identity, devices, and placement pointers are reachable
//! **even with your machine off**.
//!
//! It is **blind**. Per account root it holds:
//! - a [`DirectoryRecord`] — readable routing (root pubkey, device pubkeys, placement
//!   pointers, opaque Home routes; **no secrets**, `INV-10`), and
//! - an **opaque** sealed account blob — the hex ciphertext `gaugedesk_app::account::seal_account_blob`
//!   produces. Only your devices' key opens it; the directory holds no key and never calls
//!   `gaugedesk_app::account::open_account_blob`.
//!
//! Reads are public (routing is meant to be found). **Writes are signed and fail-closed**:
//! only the holder of the root key may publish under it — the signature is verified against
//! the record's *own* `root_pubkey` by [`put_verifies`], so a public directory cannot be
//! hijacked by a stranger overwriting your routing.
//!
//! **One contract with the hosted directory.** The wire types and the verifier are the
//! platform's `gaugedesk-directory-protocol` crate — the same crate the hosted edge Worker
//! compiles to wasm — and the host rules below are the Worker's, so what this repository
//! lets anyone audit is what the hosted directory enforces:
//!
//! - `PUT /directory/:root` takes a [`SignedDirectoryPut`]. An unparseable body, a
//!   non-positive `generation`, or a record whose `root_pubkey` is not the path's root is
//!   `422`; a signature that does not verify under that root is `401`.
//! - **Replay fencing.** A publish must advance the root's generation by exactly one (the
//!   first is `1`); anything else is `409`. The byte-identical publish of an entry already
//!   admitted is idempotent (`204`, nothing appended), so a retry after a lost response is
//!   safe, while an old entry cannot be replayed over a newer one.
//! - **Retraction** (ADR 0153). A root-signed retraction is a generation-advancing publish
//!   like any other; a root whose latest entry is one reads as `410 Gone`, with the empty
//!   retraction as the body.
//! - `GET /directory/:root` serves the latest admitted signed put **byte for byte as it was
//!   published** (`200`), so a reader can verify the root signature over it itself. An
//!   unknown root is `404`, and a path that is not an account root is `404`.
//!
//! Durable, append-only backing (`INV-6`): each admitted publish appends its exact body to
//! the reserved `directory` scope, and [`BlindDirectory::open`] rehydrates by folding that
//! log in order (`INV-5`), so the directory survives a restart. Entries written before
//! signed puts were stored — a bare entry with no signature — rehydrate as generation `0`
//! and are served in that shape, which the client reads as unverifiable routing.
//! [`BlindDirectory::new`] is the pure in-memory directory (dev/test).
//!
//! [ADR 0054]: ../../specs/decisions/0054-account-directory-and-sealed-sync.md

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::put,
    Router,
};
use sha2::{Digest, Sha256};

use gaugedesk_store::Store;

pub use gaugedesk_directory_protocol::{
    is_retraction, put_verifies, retraction_entry, signing_bytes, DirectoryEntry, DirectoryRecord,
    SignedDirectoryPut,
};

/// The reserved scope the durable directory log lives in.
pub const DIRECTORY_SCOPE: &str = "directory";
/// The record kind each admitted publish appends (one per publish; the latest per root wins).
pub const DIRECTORY_ENTRY_KIND: &str = "directory_entry";
/// The largest publish body accepted, the same bound the hosted Worker applies.
pub const MAX_PUT_BYTES: usize = 2 * 1024 * 1024;

/// Whether `root` has the shape of an account root public key: a SEC1 point in lowercase
/// hex, compressed or uncompressed. The hosted Worker applies the same pattern to the path.
pub fn is_account_root(root: &str) -> bool {
    let prefixed = root.starts_with("02") || root.starts_with("03") || root.starts_with("04");
    let hex = root.len().checked_sub(2).map(|_| &root[2..]).unwrap_or("");
    prefixed
        && (64..=128).contains(&hex.len())
        && hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The latest admitted value for one root, exactly as it was published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stored {
    /// The publish body, served verbatim so the root signature still covers it.
    pub body: String,
    /// The entry the body carries.
    pub entry: DirectoryEntry,
}

/// What a publish did to a root's log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    /// Appended: the generation advanced by exactly one.
    Advanced,
    /// Already admitted byte for byte; nothing appended.
    Idempotent,
    /// Refused: the generation did not advance by exactly one.
    Conflict,
}

#[derive(Default)]
struct RootLog {
    latest: Option<Stored>,
    digests: HashSet<String>,
}

impl RootLog {
    fn generation(&self) -> u64 {
        self.latest.as_ref().map_or(0, |s| s.entry.generation)
    }

    fn fold(&mut self, body: String, entry: DirectoryEntry) {
        self.digests.insert(digest(&body));
        self.latest = Some(Stored { body, entry });
    }
}

fn digest(body: &str) -> String {
    Sha256::digest(body.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A stored log row: a signed put, or a bare entry written before puts were stored whole.
fn parse_row(row: &str) -> Option<DirectoryEntry> {
    if let Ok(put) = serde_json::from_str::<SignedDirectoryPut>(row) {
        return Some(put.entry);
    }
    serde_json::from_str::<DirectoryEntry>(row).ok()
}

/// The blind directory store — keyed by account root pubkey. An in-memory read cache over an
/// **optional durable, append-only log** (`store`): `new` is pure in-memory (dev/test); `open`
/// is durable and rehydrated. `Clone` shares both (the router takes it by value).
#[derive(Clone, Default)]
pub struct BlindDirectory {
    roots: Arc<Mutex<HashMap<String, RootLog>>>,
    /// The durable backing (`INV-6`). `None` ⇒ pure in-memory. Behind `Arc<Mutex<…>>` because
    /// `append_record` needs `&mut`, and clones must share one writer (`INV-7`).
    store: Option<Arc<Mutex<Store>>>,
}

impl BlindDirectory {
    /// A pure in-memory directory (dev/test) — no durability across a restart.
    pub fn new() -> Self {
        Self::default()
    }

    /// A **durable** directory over `store`, rehydrating from the append-only log in order,
    /// so a later publish for a root supersedes an earlier one (`INV-5`).
    pub fn open(store: Store) -> Self {
        let mut roots: HashMap<String, RootLog> = HashMap::new();
        if let Ok(rows) = store.records(DIRECTORY_SCOPE, DIRECTORY_ENTRY_KIND) {
            for row in rows {
                if let Some(entry) = parse_row(&row) {
                    roots
                        .entry(entry.directory.root_pubkey.clone())
                        .or_default()
                        .fold(row, entry);
                }
            }
        }
        Self {
            roots: Arc::new(Mutex::new(roots)),
            store: Some(Arc::new(Mutex::new(store))),
        }
    }

    /// The latest admitted value for `root`, retraction included.
    pub fn get(&self, root: &str) -> Option<Stored> {
        self.roots
            .lock()
            .unwrap()
            .get(root)
            .and_then(|log| log.latest.clone())
    }

    /// Admit a verified publish whose exact body is `body`, applying the generation fence.
    /// The caller has already checked the path, the shape, and the signature.
    pub fn admit(&self, put: &SignedDirectoryPut, body: &str) -> Disposition {
        let root = put.entry.directory.root_pubkey.clone();
        let mut roots = self.roots.lock().unwrap();
        let log = roots.entry(root).or_default();
        if log.digests.contains(&digest(body)) {
            return Disposition::Idempotent;
        }
        if put.entry.generation != log.generation() + 1 {
            return Disposition::Conflict;
        }
        // Durable append first (`INV-6`: the log is the record), then the read cache. The
        // root's lock is held across both, so two publishes for one root cannot both advance.
        if let Some(store) = self.store.as_ref() {
            let _ =
                store
                    .lock()
                    .unwrap()
                    .append_record(DIRECTORY_SCOPE, DIRECTORY_ENTRY_KIND, body);
        }
        log.fold(body.to_string(), put.entry.clone());
        Disposition::Advanced
    }

    /// How many roots have ever been published.
    pub fn len(&self) -> usize {
        self.roots.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The directory HTTP surface: `PUT /directory/:root` (signed publish) and
/// `GET /directory/:root` (public read).
pub fn directory_router(dir: BlindDirectory) -> Router {
    Router::new()
        .route("/directory/:root", put(publish).get(fetch))
        .with_state(dir)
}

fn text(status: StatusCode, message: &'static str) -> Response {
    (status, message).into_response()
}

fn signed_json(status: StatusCode, body: String) -> Response {
    (
        status,
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response()
}

/// Publish for `root` — **fail-closed**, and fenced by generation.
async fn publish(
    State(dir): State<BlindDirectory>,
    Path(root): Path<String>,
    body: String,
) -> Response {
    if !is_account_root(&root) {
        return text(StatusCode::NOT_FOUND, "not found");
    }
    if body.len() > MAX_PUT_BYTES {
        return text(StatusCode::PAYLOAD_TOO_LARGE, "directory publish too large");
    }
    let put = match serde_json::from_str::<SignedDirectoryPut>(&body) {
        Ok(put) if put.entry.generation > 0 && put.entry.directory.root_pubkey == root => put,
        _ => return text(StatusCode::UNPROCESSABLE_ENTITY, "directory root mismatch"),
    };
    if !put_verifies(&put) {
        return text(StatusCode::UNAUTHORIZED, "invalid root signature");
    }
    match dir.admit(&put, &body) {
        Disposition::Advanced | Disposition::Idempotent => StatusCode::NO_CONTENT.into_response(),
        Disposition::Conflict => text(
            StatusCode::CONFLICT,
            "directory generation must advance exactly once",
        ),
    }
}

/// The latest admitted publish for `root`, verbatim; `410` once it is a retraction.
async fn fetch(State(dir): State<BlindDirectory>, Path(root): Path<String>) -> Response {
    if !is_account_root(&root) {
        return text(StatusCode::NOT_FOUND, "not found");
    }
    match dir.get(&root) {
        None => text(StatusCode::NOT_FOUND, "not found"),
        Some(stored) if is_retraction(&stored.entry) => signed_json(StatusCode::GONE, stored.body),
        Some(stored) => signed_json(StatusCode::OK, stored.body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use gaugedesk_core::signature::SigningKey;

    fn entry_for(root: &str, generation: u64, sealed_blob: &str) -> DirectoryEntry {
        DirectoryEntry {
            generation,
            directory: DirectoryRecord {
                root_pubkey: root.to_string(),
                device_pubkeys: vec!["dev-1".into(), "dev-2".into()],
                placement_pointers: vec!["relay://relay.invalid/p1".into()],
                home_routes: vec![],
            },
            // An opaque sealed blob (shape only — the directory never inspects it).
            sealed_blob: sealed_blob.into(),
            retracted: false,
        }
    }

    fn signed_body(sk: &SigningKey, entry: &DirectoryEntry) -> String {
        let put = SignedDirectoryPut {
            entry: entry.clone(),
            signature: sk.sign(&signing_bytes(entry).unwrap()),
        };
        serde_json::to_string(&put).unwrap()
    }

    async fn put_status(dir: &BlindDirectory, root: &str, body: String) -> StatusCode {
        publish(State(dir.clone()), Path(root.to_string()), body)
            .await
            .status()
    }

    async fn read(dir: &BlindDirectory, root: &str) -> (StatusCode, String) {
        let response = fetch(State(dir.clone()), Path(root.to_string())).await;
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn key(seed: u8) -> (SigningKey, String) {
        let sk = SigningKey::from_seed(&[seed; 32]).unwrap();
        let root = sk.public_key().as_str().to_string();
        (sk, root)
    }

    #[test]
    fn a_signing_key_root_is_an_account_root_and_other_paths_are_not() {
        let (_, root) = key(3);
        assert!(is_account_root(&root));
        assert!(!is_account_root("nobody"));
        assert!(!is_account_root(&root.to_uppercase()));
        assert!(!is_account_root(""));
    }

    #[tokio::test]
    async fn a_signed_publish_is_served_back_byte_for_byte() {
        let (sk, root) = key(3);
        let dir = BlindDirectory::new();
        let body = signed_body(&sk, &entry_for(&root, 1, "deadbeefcafe"));

        assert_eq!(
            put_status(&dir, &root, body.clone()).await,
            StatusCode::NO_CONTENT
        );
        let (status, served) = read(&dir, &root).await;
        assert_eq!(status, StatusCode::OK);
        // The whole signed put, verbatim, so a reader can check the signature itself.
        assert_eq!(served, body);
        let put: SignedDirectoryPut = serde_json::from_str(&served).unwrap();
        assert!(put_verifies(&put));
    }

    #[tokio::test]
    async fn a_wrongly_signed_write_is_refused_and_stores_nothing() {
        let (_, root) = key(3);
        let (attacker, _) = key(9);
        let dir = BlindDirectory::new();
        let forged = signed_body(&attacker, &entry_for(&root, 1, "00"));
        assert_eq!(
            put_status(&dir, &root, forged).await,
            StatusCode::UNAUTHORIZED
        );
        assert!(dir.is_empty());
    }

    #[tokio::test]
    async fn a_path_that_disagrees_with_the_record_is_unprocessable() {
        let (sk, root) = key(3);
        let (_, other) = key(4);
        let dir = BlindDirectory::new();
        let body = signed_body(&sk, &entry_for(&root, 1, "00"));
        assert_eq!(
            put_status(&dir, &other, body).await,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert!(dir.is_empty());
    }

    #[tokio::test]
    async fn a_malformed_body_or_a_zero_generation_is_unprocessable() {
        let (sk, root) = key(3);
        let dir = BlindDirectory::new();
        assert_eq!(
            put_status(&dir, &root, "{".into()).await,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        let zero = signed_body(&sk, &entry_for(&root, 0, "00"));
        assert_eq!(
            put_status(&dir, &root, zero).await,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert!(dir.is_empty());
    }

    #[tokio::test]
    async fn a_path_that_is_not_an_account_root_is_not_found() {
        let dir = BlindDirectory::new();
        assert_eq!(read(&dir, "nobody").await.0, StatusCode::NOT_FOUND);
        assert_eq!(
            put_status(&dir, "nobody", "{}".into()).await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn fetching_an_unknown_root_is_not_found() {
        let (_, root) = key(3);
        assert_eq!(
            read(&BlindDirectory::new(), &root).await.0,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn the_generation_advances_exactly_once_and_an_old_entry_cannot_be_replayed() {
        let (sk, root) = key(3);
        let dir = BlindDirectory::new();
        let first = signed_body(&sk, &entry_for(&root, 1, "0001"));
        let second = signed_body(&sk, &entry_for(&root, 2, "0002"));

        // The first publish must be generation one, and a skip is refused.
        let skip = signed_body(&sk, &entry_for(&root, 2, "ffff"));
        assert_eq!(put_status(&dir, &root, skip).await, StatusCode::CONFLICT);
        assert!(dir.is_empty() || dir.get(&root).is_none());

        assert_eq!(
            put_status(&dir, &root, first.clone()).await,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            put_status(&dir, &root, second.clone()).await,
            StatusCode::NO_CONTENT
        );

        // A retry of an admitted body is idempotent, even the older one, and changes nothing.
        assert_eq!(put_status(&dir, &root, first).await, StatusCode::NO_CONTENT);
        assert_eq!(read(&dir, &root).await.1, second);

        // A different entry at an old generation is a replay, and is refused.
        let stale = signed_body(&sk, &entry_for(&root, 1, "beef"));
        assert_eq!(put_status(&dir, &root, stale).await, StatusCode::CONFLICT);
        assert_eq!(read(&dir, &root).await.1, second);
    }

    #[tokio::test]
    async fn a_retracted_root_is_gone_and_can_publish_again() {
        let (sk, root) = key(3);
        let dir = BlindDirectory::new();
        let first = signed_body(&sk, &entry_for(&root, 1, "0001"));
        assert_eq!(put_status(&dir, &root, first).await, StatusCode::NO_CONTENT);

        let retraction = signed_body(&sk, &retraction_entry(root.clone(), 2));
        assert_eq!(
            put_status(&dir, &root, retraction.clone()).await,
            StatusCode::NO_CONTENT
        );
        let (status, served) = read(&dir, &root).await;
        assert_eq!(status, StatusCode::GONE);
        // The retraction is the body: empty routing, and the generation to re-publish from.
        assert_eq!(served, retraction);

        let again = signed_body(&sk, &entry_for(&root, 3, "0003"));
        assert_eq!(
            put_status(&dir, &root, again.clone()).await,
            StatusCode::NO_CONTENT
        );
        assert_eq!(read(&dir, &root).await, (StatusCode::OK, again));
    }

    #[tokio::test]
    async fn the_log_survives_a_restart_and_keeps_the_fence() {
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("directory.db");
        let db_path = db.to_str().unwrap();
        let (sk, root) = key(7);
        let first = signed_body(&sk, &entry_for(&root, 1, "0001"));
        let second = signed_body(&sk, &entry_for(&root, 2, "0002"));

        {
            let dir = BlindDirectory::open(Store::open(db_path).unwrap());
            assert_eq!(
                put_status(&dir, &root, first.clone()).await,
                StatusCode::NO_CONTENT
            );
            assert_eq!(
                put_status(&dir, &root, second.clone()).await,
                StatusCode::NO_CONTENT
            );
        }

        let restarted = BlindDirectory::open(Store::open(db_path).unwrap());
        assert_eq!(restarted.len(), 1, "one root, folded latest-wins");
        assert_eq!(read(&restarted, &root).await, (StatusCode::OK, second));
        // The fence and the idempotency record are rehydrated too, not only the latest value.
        assert_eq!(
            put_status(&restarted, &root, first).await,
            StatusCode::NO_CONTENT
        );
        let stale = signed_body(&sk, &entry_for(&root, 2, "beef"));
        assert_eq!(
            put_status(&restarted, &root, stale).await,
            StatusCode::CONFLICT
        );
        let third = signed_body(&sk, &entry_for(&root, 3, "0003"));
        assert_eq!(
            put_status(&restarted, &root, third).await,
            StatusCode::NO_CONTENT
        );
    }

    #[tokio::test]
    async fn a_bare_entry_written_before_signed_puts_rehydrates_at_generation_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("directory.db");
        let db_path = db.to_str().unwrap();
        let (sk, root) = key(8);
        let legacy = serde_json::json!({
            "directory": {
                "root_pubkey": root,
                "device_pubkeys": ["dev-1"],
                "placement_pointers": []
            },
            "sealed_blob": "00"
        })
        .to_string();
        {
            let mut store = Store::open(db_path).unwrap();
            store
                .append_record(DIRECTORY_SCOPE, DIRECTORY_ENTRY_KIND, &legacy)
                .unwrap();
        }

        let dir = BlindDirectory::open(Store::open(db_path).unwrap());
        assert_eq!(read(&dir, &root).await, (StatusCode::OK, legacy));
        // The next signed publish starts the fence at one.
        let first = signed_body(&sk, &entry_for(&root, 1, "0001"));
        assert_eq!(put_status(&dir, &root, first).await, StatusCode::NO_CONTENT);
    }
}
