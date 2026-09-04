//! Lab 08 — derive BIP32 extended private and public keys.

use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;

use crate::error::LabError;
use crate::labs::common::{derive_xpriv, master_xpriv as build_master_xpriv, parse_path};
use crate::model::ExtendedKeyReport;
use crate::LabResult;

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    Ok(build_master_xpriv(mnemonic, passphrase, network)?.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let secp = Secp256k1::new();
    let xpriv = derive_xpriv(mnemonic, passphrase, path, network)?;
    let xpub = Xpub::from_priv(&secp, &xpriv);

    Ok(ExtendedKeyReport {
        derivation_path: path.to_owned(),
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let secp = Secp256k1::new();
    let parent = Xpub::from_str(parent_xpub.trim())
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;
    let child_number = ChildNumber::from_normal_idx(index)
        .map_err(|error| LabError::InvalidPath(error.to_string()))?;

    parent
        .ckd_pub(&secp, child_number)
        .map(|child| child.to_string())
        .map_err(|error| LabError::Derivation(error.to_string()))
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let path = parse_path(path)?;
    let levels: &[ChildNumber] = path.as_ref();
    Ok(levels.iter().any(|child| child.is_hardened()))
}
