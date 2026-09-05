//! Local persistence.
//!
//! We open a single SQLite file and hand the same connection to two different things:
//!
//! 1. BDK itself, which creates and manages its own tables in it (via `bdk_wallet`'s built-in
//!    `rusqlite` [`WalletPersister`] impl) to store descriptors, the block/tx graph, address
//!    indices and UTXOs every time [`bdk_wallet::Wallet::persist`] is called.
//! 2. One extra table we own, `wallet_seed`, which holds the wallet's BIP39 mnemonic. This is the
//!    one secret the wallet needs to be able to sign again after a restart; everything else BDK
//!    tracks in its own tables is re-derived from chain data during `sync`.
//!
//! Keeping both in the same file means "restart the wallet" is just "reopen this one file".

use bdk_wallet::rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;

pub fn open(path: impl AsRef<Path>) -> Result<Connection> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS wallet_seed (
            id          INTEGER PRIMARY KEY CHECK (id = 1),
            mnemonic    TEXT NOT NULL,
            created_at  INTEGER NOT NULL
        );",
    )?;
    Ok(conn)
}

/// Returns the stored mnemonic, if a wallet has already been created in this database.
pub fn load_mnemonic(conn: &Connection) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT mnemonic FROM wallet_seed WHERE id = 1")?;
    let mut rows = stmt.query([])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

/// Persists a freshly generated mnemonic. Only ever called once per database (on first use).
pub fn save_mnemonic(conn: &Connection, mnemonic: &str) -> Result<()> {
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    conn.execute(
        "INSERT INTO wallet_seed (id, mnemonic, created_at) VALUES (1, ?1, ?2)",
        params![mnemonic, created_at],
    )?;
    Ok(())
}
