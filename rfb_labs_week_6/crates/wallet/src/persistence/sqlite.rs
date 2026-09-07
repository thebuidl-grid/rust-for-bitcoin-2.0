use std::path::{Path, PathBuf};

use crate::{
    error::{WalletError, WalletResult},
    persistence::WalletStore,
};

/// Owns the SQLite location. The BDK SQLite connection will live here so
/// database details remain outside wallet and CLI code.
pub struct SqliteStore {
    path: PathBuf,
}

impl SqliteStore {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl WalletStore for SqliteStore {
    fn open(path: &Path) -> WalletResult<Self> {
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    fn initialize(&mut self) -> WalletResult<()> {
        Err(WalletError::NotImplemented("SQLite persistence"))
    }
}
