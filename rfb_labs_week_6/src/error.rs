use std::fmt;

/// A single error type for the whole wallet binary.
///
/// The assignment asks for "meaningful errors", not a deep hierarchy, so this stays small: the
/// two dependencies we call directly and constantly (Bitcoin Core RPC, SQLite) get their own
/// variant with a `From` impl so `?` works everywhere; every other library error (BDK's
/// descriptor/tx-building/signing/loading errors, address/amount parsing, ...) is converted with
/// `.map_err(WalletError::app)` at the call site into `App`, since those error types are numerous
/// but all implement `Display` well enough to just carry their message forward.
#[derive(Debug)]
pub enum WalletError {
    Rpc(bitcoincore_rpc::Error),
    Sqlite(bdk_wallet::rusqlite::Error),
    Io(std::io::Error),
    /// Anything else: a wrapped library error message, or a problem with how *we* used the
    /// wallet (bad CLI input, wallet not found, insufficient funds, ...).
    App(String),
}

impl WalletError {
    pub fn app(e: impl fmt::Display) -> Self {
        WalletError::App(e.to_string())
    }
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::Rpc(e) => write!(f, "Bitcoin Core RPC error: {e}"),
            WalletError::Sqlite(e) => write!(f, "local database error: {e}"),
            WalletError::Io(e) => write!(f, "I/O error: {e}"),
            WalletError::App(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for WalletError {}

impl From<bitcoincore_rpc::Error> for WalletError {
    fn from(e: bitcoincore_rpc::Error) -> Self {
        WalletError::Rpc(e)
    }
}
impl From<bdk_wallet::rusqlite::Error> for WalletError {
    fn from(e: bdk_wallet::rusqlite::Error) -> Self {
        WalletError::Sqlite(e)
    }
}
impl From<std::io::Error> for WalletError {
    fn from(e: std::io::Error) -> Self {
        WalletError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, WalletError>;
