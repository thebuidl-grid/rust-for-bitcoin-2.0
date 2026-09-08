//! Lab 08 — derive BIP32 extended private and public keys.

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;
use std::str::FromStr;

use crate::error::LabError;
use crate::model::ExtendedKeyReport;
use crate::LabResult;

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic_str: &str, passphrase: &str, network: Network) -> LabResult<String> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = m.to_seed(passphrase);
    let xprv =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;
    Ok(xprv.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic_str: &str,
    passphrase: &str,
    path_str: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = m.to_seed(passphrase);
    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;

    let path =
        DerivationPath::from_str(path_str).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let secp = Secp256k1::new();

    let derived_xprv = master
        .derive_priv(&secp, &path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    let derived_xpub = Xpub::from_priv(&secp, &derived_xprv);

    Ok(ExtendedKeyReport {
        derivation_path: path_str.to_owned(),
        xpriv: derived_xprv.to_string(),
        xpub: derived_xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub_str: &str, index: u32) -> LabResult<String> {
    let parent_xpub =
        Xpub::from_str(parent_xpub_str).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    let secp = Secp256k1::new();

    let child_number =
        ChildNumber::from_normal_idx(index).map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let child_xpub = parent_xpub
        .derive_pub(&secp, &[child_number])
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    Ok(child_xpub.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path_str: &str) -> LabResult<bool> {
    let path =
        DerivationPath::from_str(path_str).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let has_hardened = path.into_iter().any(|cn| cn.is_hardened());
    Ok(has_hardened)
}
