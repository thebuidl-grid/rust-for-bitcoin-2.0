//! Lab 08 — derive BIP32 extended private and public keys.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::Network;
use std::str::FromStr;

use crate::model::ExtendedKeyReport;
use crate::LabResult;

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    let mnemonic =
        Mnemonic::parse(mnemonic).map_err(|e| crate::LabError::InvalidMnemonic(e.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);
    let xpriv = Xpriv::new_master(network, &seed)
        .map_err(|e| crate::LabError::Derivation(e.to_string()))?;

    Ok(xpriv.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let mnemonic =
        Mnemonic::parse(mnemonic).map_err(|e| crate::LabError::InvalidMnemonic(e.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);
    let master_xpriv = Xpriv::new_master(network, &seed)
        .map_err(|e| crate::LabError::Derivation(e.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| crate::LabError::InvalidPath(e.to_string()))?;

    let derived_xpriv = master_xpriv
        .derive_priv(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path)
        .map_err(|e| crate::LabError::Derivation(e.to_string()))?;

    let xpub = Xpub::from_priv(&bitcoin::secp256k1::Secp256k1::new(), &derived_xpriv);

    Ok(ExtendedKeyReport {
        derivation_path: path.to_string(),
        xpriv: derived_xpriv.to_string(),
        xpub: xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let xpub =
        Xpub::from_str(parent_xpub).map_err(|e| crate::LabError::InvalidKey(e.to_string()))?;

    // Normal (non-hardened) child number
    if index >= 2_147_483_648 {
        return Err(crate::LabError::InvalidPath(
            "Index must be < 2^31 for normal child".to_string(),
        ));
    }

    let child_number = bitcoin::bip32::ChildNumber::from(index);

    let child_xpub = xpub
        .derive_pub(&bitcoin::secp256k1::Secp256k1::new(), &[child_number])
        .map_err(|e| crate::LabError::Derivation(e.to_string()))?;

    Ok(child_xpub.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| crate::LabError::InvalidPath(e.to_string()))?;

    Ok(derivation_path.into_iter().any(|cn| cn.is_hardened()))
}
