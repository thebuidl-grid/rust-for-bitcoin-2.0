//! Lab 08 — derive BIP32 extended private and public keys.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;

use crate::model::ExtendedKeyReport;
use crate::{LabError, LabResult};

fn seed(mnemonic: &str, passphrase: &str) -> LabResult<[u8; 64]> {
    let parsed =
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    Ok(parsed.to_seed(passphrase))
}

/// Derive the master `Xpriv` for a mnemonic/passphrase/network recovery setup.
pub(crate) fn master(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<Xpriv> {
    let seed = seed(mnemonic, passphrase)?;
    Xpriv::new_master(network, &seed).map_err(|error| LabError::Derivation(error.to_string()))
}

/// Derive the `Xpriv` selected by a full BIP32 derivation path.
pub(crate) fn derive_priv_at_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<Xpriv> {
    let master = master(mnemonic, passphrase, network)?;
    let path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let secp = Secp256k1::new();
    master
        .derive_priv(&secp, &path)
        .map_err(|error| LabError::Derivation(error.to_string()))
}

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    Ok(master(mnemonic, passphrase, network)?.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let xpriv = derive_priv_at_path(mnemonic, passphrase, path, network)?;
    let secp = Secp256k1::new();
    let xpub = Xpub::from_priv(&secp, &xpriv);

    Ok(ExtendedKeyReport {
        derivation_path: path.to_owned(),
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let parent =
        Xpub::from_str(parent_xpub).map_err(|error| LabError::InvalidKey(error.to_string()))?;
    let child_number = ChildNumber::from_normal_idx(index)
        .map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let secp = Secp256k1::new();
    let child = parent
        .derive_pub(&secp, &[child_number])
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    Ok(child.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    Ok((&path).into_iter().any(ChildNumber::is_hardened))
}
