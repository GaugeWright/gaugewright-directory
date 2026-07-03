//! gaugewright blind directory service binary (`ACCT-2`, [ADR 0054]).
//!
//! The readable routing/identity layer of the account model, hosted on the always-on relay
//! (`infra/relay/`). Per account root it serves a readable `DirectoryRecord` + an opaque
//! sealed account blob it cannot open (`INV-10`); reads are public, writes are signed by the
//! root key (fail-closed). See [`gaugewright_directory`].
//!
//! Run alongside the rendezvous broker on the relay host:
//!   GAUGEWRIGHT_DIRECTORY_ADDR=0.0.0.0:7901 gaugewright-directory
//! Set `GAUGEWRIGHT_DIRECTORY_DB=/path/dir.db` for **durable** routing that survives a relay
//! restart (`ACCT-2`); unset keeps the pure in-memory cache (dev). An optional
//! `GAUGEWRIGHT_DIRECTORY_READY` file is touched once the listener is bound, so a
//! container/orchestrator healthcheck can gate on it.

use gaugewright_directory::{directory_router, BlindDirectory};
use gaugewright_store::Store;

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .try_init();

    let addr = std::env::var("GAUGEWRIGHT_DIRECTORY_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:7901".to_string());
    let ready = std::env::var("GAUGEWRIGHT_DIRECTORY_READY").ok();

    // Durable when a DB path is configured (the deployed relay), else in-memory (dev).
    let directory = match std::env::var("GAUGEWRIGHT_DIRECTORY_DB")
        .ok()
        .filter(|p| !p.is_empty())
    {
        Some(path) => {
            let store = Store::open(&path).expect("open directory db");
            println!("[directory] durable backing: {path}");
            BlindDirectory::open(store)
        }
        None => BlindDirectory::new(),
    };
    let app = directory_router(directory);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind directory");
    if let Some(path) = ready.as_deref() {
        let _ = std::fs::write(path, b"");
    }
    println!("[directory] blind directory listening on {addr}");
    axum::serve(listener, app).await.expect("serve directory");
}
