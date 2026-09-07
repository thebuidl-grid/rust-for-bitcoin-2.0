mod sqlite;

pub use sqlite::SqliteStore;

use std::path::Path;

use crate::error::WalletResult;

/// Lifecycle boundary for local wallet state.
pub trait WalletStore: Sized {
    fn open(path: &Path) -> WalletResult<Self>;
    fn initialize(&mut self) -> WalletResult<()>;
}
