use std::path::Path;

use bdk_wallet::rusqlite::Connection;

use crate::error::WalletError;

/// Opens (creating if needed) the SQLite database backing wallet state.
/// `bdk_wallet`'s `rusqlite` feature manages its own schema inside this
/// connection — there's nothing else to migrate or initialize here.
pub fn open_db(path: &Path) -> Result<Connection, WalletError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| WalletError::Persistence(e.to_string()))?;
        }
    }
    Connection::open(path).map_err(|e| WalletError::Persistence(e.to_string()))
}