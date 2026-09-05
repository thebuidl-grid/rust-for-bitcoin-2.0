//! Lab 08 — derive BIP32 extended private and public keys.

use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::Network;
use std::str::FromStr;

use crate::model::ExtendedKeyReport;
use crate::LabResult;

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    let seed_hex = crate::labs::lab07_bip39::mnemonic_seed_hex(mnemonic, passphrase)?;

    let seed =
        hex::decode(seed_hex).map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let xpriv = Xpriv::new_master(network, &seed)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    Ok(xpriv.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let seed_hex = crate::labs::lab07_bip39::mnemonic_seed_hex(mnemonic, passphrase)?;

    let seed =
        hex::decode(seed_hex).map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let secp = bitcoin::secp256k1::Secp256k1::new();

    let derived = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let xpub = Xpub::from_priv(&secp, &derived);

    Ok(ExtendedKeyReport {
        derivation_path: path.to_owned(),
        xpriv: derived.to_string(),
        xpub: xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let xpub = Xpub::from_str(parent_xpub)
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let secp = bitcoin::secp256k1::Secp256k1::new();

    let child = xpub
        .derive_pub(&secp, &[bitcoin::bip32::ChildNumber::Normal { index }])
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    Ok(child.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let derivation_path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    Ok(derivation_path.into_iter().any(|child| child.is_hardened()))
}
