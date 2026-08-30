//! Lab 08 — derive BIP32 extended private and public keys.

use bitcoin::Network;

use crate::model::ExtendedKeyReport;
use crate::LabResult;

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    let parsed = bip39::Mnemonic::parse(mnemonic)
        .map_err(|e| crate::error::LabError::InvalidMnemonic(e.to_string()))?;
    let seed = parsed.to_seed(passphrase);
    let master = bitcoin::bip32::Xpriv::new_master(network, &seed)
        .map_err(|e| crate::error::LabError::Derivation(e.to_string()))?;
    Ok(master.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    use std::str::FromStr;
    let derivation_path = bitcoin::bip32::DerivationPath::from_str(path)
        .map_err(|e| crate::error::LabError::InvalidPath(e.to_string()))?;

    let master_str = master_xpriv(mnemonic, passphrase, network)?;
    let master = bitcoin::bip32::Xpriv::from_str(&master_str)
        .map_err(|e| crate::error::LabError::Derivation(e.to_string()))?;

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let child_xpriv = master.derive_priv(&secp, &derivation_path)
        .map_err(|e| crate::error::LabError::Derivation(e.to_string()))?;
    let child_xpub = bitcoin::bip32::Xpub::from_priv(&secp, &child_xpriv);

    Ok(ExtendedKeyReport {
        derivation_path: path.to_owned(),
        xpriv: child_xpriv.to_string(),
        xpub: child_xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    use std::str::FromStr;
    let parent = bitcoin::bip32::Xpub::from_str(parent_xpub)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    let child_number = bitcoin::bip32::ChildNumber::from_normal_idx(index)
        .map_err(|e| crate::error::LabError::Derivation(e.to_string()))?;

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let child = parent
        .derive_pub(&secp, &[child_number])
        .map_err(|e| crate::error::LabError::Derivation(e.to_string()))?;
    Ok(child.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    use std::str::FromStr;
    let derivation_path = bitcoin::bip32::DerivationPath::from_str(path)
        .map_err(|e| crate::error::LabError::InvalidPath(e.to_string()))?;
    let path_vec: Vec<bitcoin::bip32::ChildNumber> = derivation_path.into();
    let contains_hardened = path_vec.iter().any(|child| child.is_hardened());
    Ok(contains_hardened)
}

