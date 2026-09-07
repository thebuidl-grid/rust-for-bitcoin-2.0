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

    #[error(
        "could not connect to Bitcoin Core at `{url}`: {source}\n\
         If you are using Polar, make sure the workspace is running and its Bitcoin Core node is started, then copy that node's RPC host, port, username, and password into MUF_RPC_URL, MUF_RPC_USER, and MUF_RPC_PASSWORD."
    )]
    BitcoinCoreConnection {
        url: String,
        #[source]
        source: bitcoincore_rpc::Error,
    },

    #[error(
        "Bitcoin Core network mismatch at `{url}`: wallet expects `{expected}`, but the node reports `{actual}`. Start or select a Polar/Bitcoin Core node on `{expected}`, or correct MUF_NETWORK."
    )]
    BitcoinCoreNetworkMismatch {
        url: String,
        expected: bitcoin::Network,
        actual: bitcoin::Network,
    },

    #[error("wallet chain update error: {0}")]
    ChainUpdate(#[from] bdk_wallet::chain::local_chain::ApplyHeaderError),

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
    use bitcoin::Network;

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

    #[test]
    fn connection_error_explains_how_to_check_polar_configuration() {
        let error = WalletError::BitcoinCoreConnection {
            url: "http://127.0.0.1:18443".into(),
            source: bitcoincore_rpc::Error::ReturnedError("authentication failed".into()),
        };
        let message = error.to_string();

        assert!(message.contains("Polar"));
        assert!(message.contains("Bitcoin Core node is started"));
        assert!(message.contains("MUF_RPC_URL"));
        assert!(message.contains("MUF_RPC_USER"));
        assert!(message.contains("MUF_RPC_PASSWORD"));
    }

    #[test]
    fn network_mismatch_names_both_networks() {
        let error = WalletError::BitcoinCoreNetworkMismatch {
            url: "http://127.0.0.1:18443".into(),
            expected: Network::Regtest,
            actual: Network::Bitcoin,
        };
        let message = error.to_string();

        assert!(message.contains("regtest"));
        assert!(message.contains("bitcoin"));
    }
}
