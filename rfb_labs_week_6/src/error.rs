use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVar(&'static str),
    #[error("invalid BITCOIN_NETWORK value: '{0}' (expected 'regtest' or 'testnet')")]
    InvalidNetwork(String),
}

// TODO(stage 4+): WalletError, NodeError, TxError
