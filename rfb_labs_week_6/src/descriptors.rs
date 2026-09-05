//! Derive BIP84/BIP86 external + internal output descriptors from a BIP39
//! mnemonic, and turn them into a loaded/created `bdk_wallet::Wallet`.

use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use bdk_wallet::bitcoin::bip32::DerivationPath;
use bdk_wallet::bitcoin::secp256k1::Secp256k1;
use bdk_wallet::bitcoin::{Network, NetworkKind};
use bdk_wallet::descriptor::IntoWalletDescriptor;
use bdk_wallet::keys::bip39::{Language, Mnemonic};
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, PersistedWallet, Wallet as WalletBuilder, descriptor};

use crate::config::{Config, DescriptorKind};

/// A wallet loaded from (and kept in sync with) our SQLite store. `Wallet`
/// itself is just the builder namespace (`Wallet::create`/`Wallet::load`);
/// the live, persistable value is `PersistedWallet<Connection>`.
pub type Wallet = PersistedWallet<Connection>;

/// A matched pair of `tprv`/`xprv`-bearing output descriptors: one for the
/// external (receive) keychain, one for the internal (change) keychain.
pub struct WalletDescriptors {
    pub external: String,
    pub internal: String,
}

fn network_kind(network: Network) -> NetworkKind {
    match network {
        Network::Bitcoin => NetworkKind::Main,
        _ => NetworkKind::Test,
    }
}

/// SLIP-44 coin type used at the `coin'` level of the path: 0 for mainnet,
/// 1 (the shared testnet/signet/regtest coin type) otherwise.
fn coin_type(network: Network) -> u32 {
    match network {
        Network::Bitcoin => 0,
        _ => 1,
    }
}

/// Derive the external/internal descriptor pair for `config`'s mnemonic,
/// network, account, and chosen script type (BIP84 `wpkh` or BIP86 `tr`).
pub fn derive_descriptors(config: &Config) -> Result<WalletDescriptors> {
    let phrase = config.mnemonic.as_deref().ok_or_else(|| {
        anyhow!("no MNEMONIC configured; run `init` first (it generates and saves one to .env)")
    })?;
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, phrase)
        .context("MNEMONIC is not a valid BIP39 mnemonic")?;
    let passphrase = (!config.passphrase.is_empty()).then(|| config.passphrase.clone());
    let mnemonic_with_passphrase = (mnemonic, passphrase);

    let purpose = match config.descriptor_kind {
        DescriptorKind::Wpkh => 84,
        DescriptorKind::Tr => 86,
    };
    let coin = coin_type(config.network);
    let account = config.account;
    let external_path = DerivationPath::from_str(&format!("m/{purpose}h/{coin}h/{account}h/0"))?;
    let internal_path = DerivationPath::from_str(&format!("m/{purpose}h/{coin}h/{account}h/1"))?;

    let secp = Secp256k1::new();
    let kind = network_kind(config.network);

    let (external, internal) = match config.descriptor_kind {
        DescriptorKind::Wpkh => {
            let (external_desc, external_keymap) =
                descriptor!(wpkh((mnemonic_with_passphrase.clone(), external_path)))?
                    .into_wallet_descriptor(&secp, kind)?;
            let (internal_desc, internal_keymap) =
                descriptor!(wpkh((mnemonic_with_passphrase, internal_path)))?
                    .into_wallet_descriptor(&secp, kind)?;
            (
                external_desc.to_string_with_secret(&external_keymap),
                internal_desc.to_string_with_secret(&internal_keymap),
            )
        }
        DescriptorKind::Tr => {
            let (external_desc, external_keymap) =
                descriptor!(tr((mnemonic_with_passphrase.clone(), external_path)))?
                    .into_wallet_descriptor(&secp, kind)?;
            let (internal_desc, internal_keymap) =
                descriptor!(tr((mnemonic_with_passphrase, internal_path)))?
                    .into_wallet_descriptor(&secp, kind)?;
            (
                external_desc.to_string_with_secret(&external_keymap),
                internal_desc.to_string_with_secret(&internal_keymap),
            )
        }
    };

    Ok(WalletDescriptors { external, internal })
}

/// Open (creating if needed) the wallet's local SQLite state file.
pub fn open_db(config: &Config) -> Result<Connection> {
    Connection::open(&config.db_path)
        .with_context(|| format!("failed to open wallet database at {:?}", config.db_path))
}

/// Load the wallet from `db` if it was created before, otherwise create it
/// fresh from `config`'s descriptors and persist that first state.
pub fn load_or_create_wallet(config: &Config, db: &mut Connection) -> Result<Wallet> {
    let descriptors = derive_descriptors(config)?;

    let loaded = WalletBuilder::load()
        .descriptor(KeychainKind::External, Some(descriptors.external.clone()))
        .descriptor(KeychainKind::Internal, Some(descriptors.internal.clone()))
        .extract_keys()
        .check_network(config.network)
        .load_wallet(db)
        .context("failed to load existing wallet state from the database")?;

    match loaded {
        Some(wallet) => Ok(wallet),
        None => WalletBuilder::create(descriptors.external, descriptors.internal)
            .network(config.network)
            .create_wallet(db)
            .context("failed to create a new wallet"),
    }
}
