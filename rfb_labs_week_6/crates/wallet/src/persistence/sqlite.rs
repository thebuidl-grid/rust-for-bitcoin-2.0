use std::path::{Path, PathBuf};

use bdk_wallet::rusqlite::Connection;

use crate::error::WalletResult;

/// Owns the SQLite location. The BDK-compatible connection lives here so
/// database details remain outside wallet and CLI code.
pub struct SqliteStore {
    path: PathBuf,
    connection: Connection,
}

impl SqliteStore {
    pub fn open(path: &Path) -> WalletResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            connection,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}
