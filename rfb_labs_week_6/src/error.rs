use std::path::PathBuf;

/// Every fallible operation in the wallet returns this error type. The variants
/// are grouped by the layer that produces them (config, node, wallet, tx) so a
/// caller can react differently to, say, a missing node versus a bad address.
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("wallet database not found at {0}; run `rfbwallet init` first")]
    WalletNotInitialized(PathBuf),

    #[error("a wallet database already exists at {0}; delete it or point DB_PATH elsewhere")]
    WalletAlreadyExists(PathBuf),

    #[error("descriptor error: {0}")]
    Descriptor(String),

    #[error("bip39 mnemonic error: {0}")]
    Mnemonic(String),

    #[error("could not open or load the wallet: {0}")]
    Load(String),

    #[error("node connection error: {0}")]
    Connect(String),

    #[error("could not build the transaction: {0}")]
    BuildTx(String),

    #[error("insufficient funds: need {need} sat, spendable balance is {available} sat")]
    InsufficientFunds { need: u64, available: u64 },

    #[error("signing error: {0}")]
    Sign(String),

    #[error("invalid address: {0}")]
    Address(String),

    #[error("this command is only available on regtest")]
    RegtestOnly,

    #[error("bitcoin core rpc error: {0}")]
    Rpc(#[from] bitcoincore_rpc::Error),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] bdk_wallet::rusqlite::Error),

    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}
