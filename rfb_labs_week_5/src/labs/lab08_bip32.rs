//! Lab 08 — derive BIP32 extended private and public keys.

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;
use std::str::FromStr;

use crate::model::ExtendedKeyReport;
use crate::LabResult;

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
        .map_err(|error| crate::LabError::InvalidMnemonic(error.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);

    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    Ok(master.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
        .map_err(|error| crate::LabError::InvalidMnemonic(error.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);

    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    let secp = Secp256k1::new();

    let derived_xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let derived_xpub = Xpub::from_priv(&secp, &derived_xpriv);

    Ok(ExtendedKeyReport {
        derivation_path: path.to_owned(),
        xpriv: derived_xpriv.to_string(),
        xpub: derived_xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let parent = Xpub::from_str(parent_xpub)
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let child_number = ChildNumber::from_normal_idx(index)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let secp = Secp256k1::verification_only();

    let child = parent
        .derive_pub(&secp, &[child_number])
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    Ok(child.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    Ok(path.as_ref().iter().any(|child| child.is_hardened()))
}
