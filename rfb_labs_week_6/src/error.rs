//! Error handling for the wallet.
//!
//! Every fallible function in this crate returns [`Result<T>`]. `main.rs` is the only
//! place that prints an error and sets a non-zero exit code, so nothing below it
//! panics on bad input.
//!
//! Variants come in two flavours:
//!
//! * **Ours** — validation we perform (`MissingEnv`, `UnsupportedNetwork`, ...). We
//!   construct these by hand, and their messages tell the user what to *do*.
//! * **Wrapped** — library failures carried through `#[from]`, which generates the
//!   `From` impl that makes `?` work at the call site.

use bdk_wallet::descriptor::DescriptorError;
use bdk_wallet::error::CreateTxError;
use bdk_wallet::keys::KeyError;
use bdk_wallet::rusqlite;
use bdk_wallet::signer::SignerError;
use bdk_wallet::{CreateWithPersistError, LoadWithPersistError};

/// Crate-wide result alias, so callers write `Result<Address>` instead of
/// `std::result::Result<Address, WalletError>`.
pub type Result<T> = std::result::Result<T, WalletError>;

/// Everything that can go wrong in this wallet.
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    // ---- configuration ---------------------------------------------------
    /// A required key is absent from the environment / `.env`.
    #[error("missing required setting `{0}` — run `rfbwallet init` first, or set it in .env")]
    MissingEnv(&'static str),

    /// A key is present but its value is unusable.
    #[error("setting `{key}` has invalid value `{value}` — {reason}")]
    InvalidEnv {
        key: &'static str,
        value: String,
        reason: &'static str,
    },

    /// The assignment permits testnet/regtest only. Mainnet is refused here, on
    /// purpose, rather than being allowed to reach key derivation.
    #[error(
        "unsupported network `{0}` — this wallet runs on regtest, signet, testnet \
         or testnet4 only, never mainnet"
    )]
    UnsupportedNetwork(String),

    // ---- keys and descriptors --------------------------------------------
    #[error("invalid mnemonic: {0}")]
    Mnemonic(#[from] bdk_wallet::bip39::Error),

    /// Entropy collection or wordlist encoding failed while generating a new seed.
    #[error("could not generate a mnemonic: {0}")]
    MnemonicGeneration(String),

    /// `ExtendedKey::into_xprv` returns `None` when the key is public-only, so we
    /// cannot sign. Should be unreachable when deriving from a mnemonic.
    #[error("could not derive an extended private key from the mnemonic")]
    NoPrivateKey,

    #[error("key derivation failed: {0}")]
    Key(#[from] KeyError),

    #[error("descriptor error: {0}")]
    Descriptor(#[from] DescriptorError),

    // ---- persistence ------------------------------------------------------
    #[error("wallet database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("could not create wallet: {0}")]
    CreateWallet(#[from] CreateWithPersistError<rusqlite::Error>),

    #[error("could not load wallet: {0}")]
    LoadWallet(#[from] LoadWithPersistError<rusqlite::Error>),

    // ---- node -------------------------------------------------------------
    #[error("bitcoin node RPC error: {0}")]
    Rpc(#[from] bdk_bitcoind_rpc::bitcoincore_rpc::Error),

    /// The emitter produced a block that does not connect to our checkpoint.
    #[error("chain data does not connect to the wallet's checkpoint: {0}")]
    CannotConnect(String),

    /// A block header could not be applied to the local chain. In practice this
    /// means a reorg raced the sync loop; re-running `sync` resolves it.
    #[error("could not apply block header: {0} — re-run `sync`")]
    ApplyHeader(String),

    /// The node is on a different network than the wallet expects. Catching this
    /// early beats deriving addresses nobody on that chain can pay.
    #[error("node is on `{node}` but the wallet is configured for `{wallet}`")]
    NetworkMismatch { node: String, wallet: String },

    // ---- transactions -----------------------------------------------------
    #[error("could not build transaction: {0}")]
    BuildTx(String),

    #[error("could not sign transaction: {0}")]
    Sign(#[from] SignerError),

    /// `Wallet::sign` returning `false` means the PSBT is not fully signed. The
    /// usual cause is a wallet loaded without `.extract_keys()`, leaving it with
    /// public-only descriptors: able to watch, unable to spend.
    #[error("transaction could not be fully signed — was the wallet loaded with extract_keys()?")]
    IncompleteSignature,

    #[error("could not extract transaction from PSBT: {0}")]
    ExtractTx(String),

    // ---- io ---------------------------------------------------------------
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// `CreateTxError` is large and its Display is already descriptive, so we flatten it
// to a string rather than carrying the whole enum. Same for the PSBT extract error.
impl From<CreateTxError> for WalletError {
    fn from(e: CreateTxError) -> Self {
        Self::BuildTx(e.to_string())
    }
}

impl From<bdk_wallet::bitcoin::psbt::ExtractTxError> for WalletError {
    fn from(e: bdk_wallet::bitcoin::psbt::ExtractTxError) -> Self {
        Self::ExtractTx(e.to_string())
    }
}

impl From<bdk_wallet::chain::local_chain::CannotConnectError> for WalletError {
    fn from(e: bdk_wallet::chain::local_chain::CannotConnectError) -> Self {
        Self::CannotConnect(e.to_string())
    }
}

impl From<bdk_wallet::chain::local_chain::ApplyHeaderError> for WalletError {
    fn from(e: bdk_wallet::chain::local_chain::ApplyHeaderError) -> Self {
        Self::ApplyHeader(e.to_string())
    }
}
