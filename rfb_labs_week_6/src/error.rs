use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVar(&'static str),
    #[error("invalid BITCOIN_NETWORK value: '{0}' (expected 'regtest' or 'testnet')")]
    InvalidNetwork(String),
}

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("failed to generate a mnemonic")]
    MnemonicGeneration,
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    #[error("failed to build descriptor: {0}")]
    DescriptorBuild(String),
    #[error("wallet persistence error: {0}")]
    Persistence(String),
}

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("failed to build RPC client: {0}")]
    ClientBuild(String),
    #[error("RPC call failed: {0}")]
    Rpc(#[from] bitcoincore_rpc::Error),
    #[error("wallet sync error: {0}")]
    Sync(String),
}

#[derive(Debug, Error)]
pub enum TxError {
    #[error("failed to build transaction: {0}")]
    Build(String),
    #[error("signing did not fully finalize the PSBT")]
    NotFullyFinalized,
    #[error("failed to extract final transaction: {0}")]
    Extract(String),
    #[error("failed to broadcast transaction: {0}")]
    Broadcast(#[from] bitcoincore_rpc::Error),
    #[error("wallet persistence error: {0}")]
    Persistence(String),
}
