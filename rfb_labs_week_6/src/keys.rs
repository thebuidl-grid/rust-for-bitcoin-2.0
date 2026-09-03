//! Key generation and descriptor construction for `init`.
//!
//! We generate a fresh BIP39 mnemonic, derive its BIP32 master extended private key, and build
//! the external/internal descriptor strings by hand (`wpkh(<xprv>/84'/coin'/0'/0/*)` /
//! `.../1/*`, or the Taproot equivalent at `86'`). This is exactly what BDK's own
//! [`bdk_wallet::template::Bip84`]/[`Bip86`] templates produce internally — writing it out
//! explicitly means the descriptor strings we persist to `.env` are plain, portable text that
//! `Wallet::load` can consume directly, with no dependency on template types at load time.

use anyhow::{Context, Result};
use bdk_wallet::bitcoin::Network;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bip39::Mnemonic;

use crate::config::DescriptorKind;

/// A freshly generated wallet seed and its two keychain descriptors.
pub struct GeneratedWallet {
    pub mnemonic: Mnemonic,
    pub external_descriptor: String,
    pub internal_descriptor: String,
}

/// BIP44-style coin type: `0'` for Bitcoin mainnet, `1'` for every test network, per SLIP-44.
fn coin_type(network: Network) -> u32 {
    if network == Network::Bitcoin { 0 } else { 1 }
}

/// Generate a new 12-word mnemonic, derive its master key, and build both keychain descriptors
/// for the requested script type.
pub fn generate(network: Network, kind: DescriptorKind) -> Result<GeneratedWallet> {
    let mnemonic = Mnemonic::generate(12).context("failed to generate a BIP39 mnemonic")?;
    // No BIP39 passphrase: this is a lab wallet, and an empty passphrase keeps recovery from the
    // mnemonic alone unambiguous.
    let seed = mnemonic.to_seed("");
    let xpriv = Xpriv::new_master(network, &seed).context("failed to derive the master key")?;

    let coin = coin_type(network);
    let (purpose, script) = match kind {
        DescriptorKind::Wpkh => (84, "wpkh"),
        DescriptorKind::Tr => (86, "tr"),
    };
    let external_descriptor = format!("{script}({xpriv}/{purpose}'/{coin}'/0'/0/*)");
    let internal_descriptor = format!("{script}({xpriv}/{purpose}'/{coin}'/0'/1/*)");

    Ok(GeneratedWallet {
        mnemonic,
        external_descriptor,
        internal_descriptor,
    })
}
