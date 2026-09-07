mod keys;
mod sync;
mod transactions;

use bdk_wallet::{PersistedWallet, rusqlite::Connection};

use crate::persistence::SqliteStore;

/// Coordinates descriptor wallet operations without knowing about CLI parsing.
pub struct WalletService {
    wallet: PersistedWallet<Connection>,
    store: SqliteStore,
}

impl WalletService {
    pub fn wallet(&self) -> &bdk_wallet::Wallet {
        &self.wallet
    }

    pub fn database_path(&self) -> &std::path::Path {
        self.store.path()
    }
}
