//! Shared helpers for the BIP39/BIP32 labs (07–10).
//!
//! These keep the deterministic-derivation pipeline in one place: recovery words
//! plus an optional passphrase become a seed, the seed becomes a BIP32 master
//! key, and a textual path selects a child key.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;

use crate::error::LabError;
use crate::LabResult;

/// Parse and checksum-validate an English BIP39 mnemonic.
pub fn parse_mnemonic(mnemonic: &str) -> LabResult<Mnemonic> {
    Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))
}

/// Derive the 512-bit BIP39 seed from words and an optional passphrase.
pub fn seed_bytes(mnemonic: &str, passphrase: &str) -> LabResult<[u8; 64]> {
    Ok(parse_mnemonic(mnemonic)?.to_seed(passphrase))
}

/// Build the BIP32 master extended private key for a recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<Xpriv> {
    let seed = seed_bytes(mnemonic, passphrase)?;
    Xpriv::new_master(network, &seed).map_err(|error| LabError::Derivation(error.to_string()))
}

/// Parse a textual derivation path such as `m/84'/1'/0'/0/0`.
pub fn parse_path(path: &str) -> LabResult<DerivationPath> {
    DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))
}

/// Derive the extended private key selected by a full path.
pub fn derive_xpriv(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<Xpriv> {
    let secp = Secp256k1::new();
    let master = master_xpriv(mnemonic, passphrase, network)?;
    let path = parse_path(path)?;
    master
        .derive_priv(&secp, &path)
        .map_err(|error| LabError::Derivation(error.to_string()))
}
