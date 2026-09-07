use std::path::PathBuf;

pub type WalletResult<T> = std::result::Result<T, WalletError>;

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("wallet error: {0}")]
    Wallet(String),

    #[error("wallet descriptor error: {0}")]
    Descriptor(#[from] bdk_wallet::descriptor::DescriptorError),

    #[error("invalid BIP39 mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("a wallet is already initialized at `{0}`")]
    AlreadyInitialized(PathBuf),

    #[error("no initialized wallet was found at `{0}`; run `muf_wallet init` first")]
    NotInitialized(PathBuf),

    #[error("Bitcoin Core RPC error: {0}")]
    Rpc(#[from] bitcoincore_rpc::Error),

    #[error("wallet storage error: {0}")]
    Storage(#[from] bdk_wallet::rusqlite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("environment file error: {0}")]
    Dotenv(#[from] dotenvy::Error),

    #[error("the `{0}` operation is scaffolded but not implemented yet")]
    NotImplemented(&'static str),
}

#[cfg(test)]
mod tests {
    use super::WalletError;

    #[test]
    fn preserves_descriptor_error_details() {
        let error = WalletError::from(
            bdk_wallet::descriptor::DescriptorError::ExternalAndInternalAreTheSame,
        );

        assert_eq!(
            error.to_string(),
            "wallet descriptor error: External and internal descriptors are the same"
        );
    }
}
