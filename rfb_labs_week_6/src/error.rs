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
}

// TODO(stage 4+): NodeError, TxError
