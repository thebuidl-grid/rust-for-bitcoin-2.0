pub type WalletResult<T> = std::result::Result<T, WalletError>;

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("wallet error: {0}")]
    Wallet(String),

    #[error("Bitcoin Core RPC error: {0}")]
    Rpc(#[from] bitcoincore_rpc::Error),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("the `{0}` operation is scaffolded but not implemented yet")]
    NotImplemented(&'static str),
}
