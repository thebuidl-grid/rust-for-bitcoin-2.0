//! Lab 08 — derive BIP32 extended private and public keys.

use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;

use crate::labs::lab07_bip39::mnemonic_seed;
use crate::model::ExtendedKeyReport;
use crate::{LabError, LabResult};

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    Ok(master_key(mnemonic, passphrase, network)?.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let secp = Secp256k1::new();
    let derivation_path = parse_path(path)?;
    let xpriv = derive_xpriv(mnemonic, passphrase, &derivation_path, network)?;

    Ok(ExtendedKeyReport {
        derivation_path: format_path(&derivation_path),
        xpriv: xpriv.to_string(),
        // Neutering drops the private key but keeps the chain code, which is exactly
        // what a watch-only server needs to keep generating receive addresses.
        xpub: Xpub::from_priv(&secp, &xpriv).to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let secp = Secp256k1::new();
    let parent = Xpub::from_str(parent_xpub.trim())
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;
    // `from_normal_idx` rejects anything at or above 2^31, which is the hardened
    // range that public-only derivation cannot reach.
    let child_number = ChildNumber::from_normal_idx(index)
        .map_err(|error| LabError::Derivation(error.to_string()))?;

    parent
        .ckd_pub(&secp, child_number)
        .map(|child| child.to_string())
        .map_err(|error| LabError::Derivation(error.to_string()))
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    Ok(parse_path(path)?.into_iter().any(ChildNumber::is_hardened))
}

/// Build the BIP32 master key from the BIP39 seed.
///
/// The network only selects the serialization prefix (`xprv` or `tprv`); the key and
/// chain code themselves are network independent.
pub fn master_key(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<Xpriv> {
    let seed = mnemonic_seed(mnemonic, passphrase)?;

    Xpriv::new_master(network, &seed).map_err(|error| LabError::Derivation(error.to_string()))
}

/// Walk the master key down a full derivation path.
pub fn derive_xpriv(
    mnemonic: &str,
    passphrase: &str,
    path: &DerivationPath,
    network: Network,
) -> LabResult<Xpriv> {
    let secp = Secp256k1::new();

    master_key(mnemonic, passphrase, network)?
        .derive_priv(&secp, path)
        .map_err(|error| LabError::Derivation(error.to_string()))
}

/// Parse a textual derivation path, rejecting anything that is not BIP32 notation.
pub fn parse_path(path: &str) -> LabResult<DerivationPath> {
    DerivationPath::from_str(path.trim()).map_err(|error| LabError::InvalidPath(error.to_string()))
}

/// Render a path with the leading `m`, which `DerivationPath` itself omits.
pub fn format_path(path: &DerivationPath) -> String {
    let mut rendered = String::from("m");

    for child in path {
        rendered.push('/');
        rendered.push_str(&child.to_string());
    }

    rendered
}
