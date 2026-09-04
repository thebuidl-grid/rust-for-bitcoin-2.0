//! Lab 08 — derive BIP32 extended private and public keys.

use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;
use std::str::FromStr;

use crate::error::LabError;
use crate::model::ExtendedKeyReport;
use crate::LabResult;

fn parsed_seed(mnemonic: &str, passphrase: &str) -> LabResult<[u8; 64]> {
    let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, mnemonic)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    Ok(mnemonic.to_seed(passphrase))
}

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    let seed = parsed_seed(mnemonic, passphrase)?;
    Xpriv::new_master(network, &seed)
        .map(|key| key.to_string())
        .map_err(|e| LabError::Derivation(e.to_string()))
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let seed = parsed_seed(mnemonic, passphrase)?;
    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;
    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let secp = Secp256k1::new();
    let xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    let xpub = Xpub::from_priv(&secp, &xpriv);
    Ok(ExtendedKeyReport {
        derivation_path: path.to_owned(),
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let xpub = Xpub::from_str(parent_xpub).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    let secp = Secp256k1::new();
    xpub.derive_pub(
        &secp,
        &[bitcoin::bip32::ChildNumber::from_normal_idx(index)
            .map_err(|e| LabError::Derivation(e.to_string()))?],
    )
    .map(|key| key.to_string())
    .map_err(|e| LabError::Derivation(e.to_string()))
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let path = DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    Ok(path.into_iter().any(|child| child.is_hardened()))
}
