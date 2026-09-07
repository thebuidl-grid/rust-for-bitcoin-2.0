//! Error type for the wallet.
//!
//! Every fallible path in this crate returns [`Error`]. The binary turns it into a
//! message on stderr and a non-zero exit code, so a bad address or an unreachable
//! node produces a readable line rather than a panic and a backtrace.

use std::path::PathBuf;

use bdk_wallet::descriptor::DescriptorError;
use bdk_wallet::error::CreateTxError;
use bdk_wallet::signer::SignerError;
use bitcoin::Network;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error(
        "mainnet is not supported by this wallet; set RFB_NETWORK to regtest, testnet or signet"
    )]
    MainnetRefused,

    #[error(
        "no wallet database at {}; run `rfbwallet init` first",
        .0.display()
    )]
    WalletNotInitialised(PathBuf),

    #[error(
        "a wallet already exists at {}; delete it or point RFB_WALLET_DB somewhere else",
        .0.display()
    )]
    WalletAlreadyExists(PathBuf),

    #[error(
        "no mnemonic available: set RFB_MNEMONIC in your .env so the wallet can rebuild its \
         signing keys (the database only ever stores public descriptors)"
    )]
    MissingMnemonic,

    #[error(
        "wallet database is missing the `{0}` metadata row; it may have been created by an older version"
    )]
    MissingMetadata(&'static str),

    #[error("node reports chain `{node}` but the wallet is configured for `{configured}`")]
    NetworkMismatch { node: Network, configured: Network },

    #[error("address `{address}` is not valid for {network}")]
    AddressNetworkMismatch { address: String, network: Network },

    #[error("insufficient funds: wallet holds {available} spendable, transaction needs {needed}")]
    InsufficientFunds { available: String, needed: String },

    #[error("the wallet could not fully sign the transaction; it is loaded without private keys")]
    IncompleteSignature,

    #[error("`{0}` is only available on regtest")]
    RegtestOnly(&'static str),

    #[error(
        "transaction {0} was not found in the mempool, in a block the node indexed, or in this wallet"
    )]
    TransactionNotFound(bitcoin::Txid),

    #[error("cannot verify input {index}: {reason}")]
    Unverifiable { index: usize, reason: String },

    #[error("bitcoin core rpc: {0}")]
    Rpc(#[from] bitcoincore_rpc::Error),

    #[error("wallet database: {0}")]
    Sqlite(#[from] bdk_wallet::rusqlite::Error),

    #[error("descriptor: {0}")]
    Descriptor(#[from] DescriptorError),

    #[error("building transaction: {0}")]
    CreateTx(#[from] CreateTxError),

    #[error("signing: {0}")]
    Signer(#[from] SignerError),

    #[error("bip32: {0}")]
    Bip32(#[from] bitcoin::bip32::Error),

    #[error("bip39 mnemonic: {0}")]
    Bip39(#[from] bdk_wallet::bip39::Error),

    #[error("parsing address: {0}")]
    Address(#[from] bitcoin::address::ParseError),

    #[error("decoding transaction: {0}")]
    Consensus(#[from] bitcoin::consensus::encode::Error),

    #[error("hex: {0}")]
    Hex(#[from] bitcoin::hex::HexToBytesError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("{context}: {source}")]
    Wallet {
        context: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl Error {
    /// Attach a short human-readable context to any library error that does not
    /// deserve a dedicated variant.
    pub fn wallet<E>(context: &'static str, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Wallet {
            context,
            source: Box::new(source),
        }
    }
}
