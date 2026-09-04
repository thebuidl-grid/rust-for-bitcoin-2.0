/// Wallet creation and loading.
///
/// Uses `bdk_wallet` with a SQLite persistence backend.
/// The wallet uses native SegWit (wpkh) descriptors on regtest.
///
/// On first run a fresh mnemonic is generated from OS entropy, printed to
/// stdout, and then used to derive BIP-84 descriptors.  On subsequent runs
/// the wallet is loaded from the SQLite database.
///
/// If `MNEMONIC` is set in the environment the wallet is derived from that
/// BIP-39 phrase instead (BIP-84 path m/84'/1'/0').
use anyhow::{Context, Result};
use bdk_wallet::{
    bitcoin::{bip32::DerivationPath, secp256k1::Secp256k1, NetworkKind},
    descriptor,
    descriptor::IntoWalletDescriptor,
    keys::{
        bip39::{Language, Mnemonic, WordCount},
        GeneratableKey, GeneratedKey,
    },
    miniscript::Tap,
    KeychainKind, PersistedWallet, Wallet,
};
// The `descriptor!` macro expands to code that references `miniscript` by
// name, so we must bring it into scope from bdk_wallet's re-export.
use bdk_wallet::miniscript;
use log::info;
use std::str::FromStr;

use crate::config::Config;

/// Open (or create) a persisted wallet backed by SQLite.
///
/// Returns `(wallet, conn)`.  The caller must keep `conn` alive for
/// subsequent `wallet.persist(&mut conn)` calls.
pub fn open(
    cfg: &Config,
) -> Result<(
    PersistedWallet<bdk_wallet::rusqlite::Connection>,
    bdk_wallet::rusqlite::Connection,
)> {
    let network = bdk_wallet::bitcoin::Network::Regtest;
    let network_kind = NetworkKind::Test; // regtest uses testnet derivation paths

    let mut conn = bdk_wallet::rusqlite::Connection::open(&cfg.wallet_db)
        .with_context(|| format!("Cannot open wallet database '{}'", cfg.wallet_db))?;

    // Derive descriptors from a mnemonic or generate fresh ones.
    let (ext_desc, ext_keymap, int_desc, int_keymap) = descriptors_from_config(cfg, network_kind)?;

    // Try to load an existing wallet; fall back to creating a new one.
    let wallet = match Wallet::load()
        .descriptor(
            KeychainKind::External,
            Some(ext_desc.to_string_with_secret(&ext_keymap)),
        )
        .descriptor(
            KeychainKind::Internal,
            Some(int_desc.to_string_with_secret(&int_keymap)),
        )
        .extract_keys()
        .check_network(network)
        .load_wallet(&mut conn)
        .context("Failed to load wallet from database")?
    {
        Some(w) => {
            info!("Loaded existing wallet from '{}'", cfg.wallet_db);
            w
        }
        None => {
            info!("Creating new wallet in '{}'", cfg.wallet_db);
            Wallet::create(
                ext_desc.to_string_with_secret(&ext_keymap),
                int_desc.to_string_with_secret(&int_keymap),
            )
            .network(network)
            .create_wallet(&mut conn)
            .context("Failed to create wallet")?
        }
    };

    Ok((wallet, conn))
}

/// Build wpkh BIP-84 wallet descriptors from a mnemonic (env) or fresh entropy.
///
/// Returns `(external_descriptor, external_keymap, internal_descriptor, internal_keymap)`.
fn descriptors_from_config(
    cfg: &Config,
    network_kind: NetworkKind,
) -> Result<(
    bdk_wallet::descriptor::ExtendedDescriptor,
    bdk_wallet::keys::KeyMap,
    bdk_wallet::descriptor::ExtendedDescriptor,
    bdk_wallet::keys::KeyMap,
)> {
    let secp = Secp256k1::new();

    let mnemonic: Mnemonic = match &cfg.mnemonic {
        Some(phrase) => Mnemonic::parse_in(Language::English, phrase)
            .context("Invalid mnemonic in MNEMONIC environment variable")?,
        None => {
            let generated: GeneratedKey<_, Tap> =
                Mnemonic::generate((WordCount::Words12, Language::English))
                    .map_err(|e| anyhow::anyhow!("Mnemonic generation failed: {:?}", e))?;
            let m: Mnemonic = generated.into_key();
            println!(
                "\n🔑  NEW WALLET — back up this mnemonic phrase:\n\n    {m}\n\n    \
                 Set MNEMONIC=\"{m}\" in your .env to restore this wallet.\n"
            );
            m
        }
    };

    // BIP-84 derivation paths (m/84'/1'/0'/0 and m/84'/1'/0'/1)
    let ext_path = DerivationPath::from_str("m/84h/1h/0h/0").expect("static path is valid");
    let int_path = DerivationPath::from_str("m/84h/1h/0h/1").expect("static path is valid");

    let mnemonic_with_passphrase = (mnemonic, None); // no BIP-39 passphrase

    // Use the `descriptor!` macro to build wpkh descriptors from the mnemonic.
    let (ext_desc, ext_keymap) = descriptor!(wpkh((mnemonic_with_passphrase.clone(), ext_path)))?
        .into_wallet_descriptor(&secp, network_kind)
        .context("Failed to build external descriptor")?;

    let (int_desc, int_keymap) = descriptor!(wpkh((mnemonic_with_passphrase, int_path)))?
        .into_wallet_descriptor(&secp, network_kind)
        .context("Failed to build internal descriptor")?;

    Ok((ext_desc, ext_keymap, int_desc, int_keymap))
}
