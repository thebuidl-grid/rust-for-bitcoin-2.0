//! Lab 08 — derive BIP32 extended private and public keys.

use crate::model::ExtendedKeyReport;
use crate::{LabError, LabResult};
use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;
use std::str::FromStr;

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    // todo!("Lab 08: derive the seed and create the BIP32 master xpriv")
    let m =
        Mnemonic::parse(mnemonic).map_err(|_| LabError::InvalidMnemonic(mnemonic.to_string()))?;

    let seed = m.to_seed(passphrase);
    let xpriv = match Xpriv::new_master(network, &seed) {
        Ok(xpriv) => xpriv,
        Err(e) => return Err(LabError::InvalidMnemonic(e.to_string())),
    };

    Ok(xpriv.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    // todo!("Lab 08: derive an xpriv and neuter it to an xpub")

    let m =
        Mnemonic::parse(mnemonic).map_err(|_| LabError::InvalidMnemonic(mnemonic.to_string()))?;
    let seed = m.to_seed(passphrase);
    let master = match Xpriv::new_master(network, &seed) {
        Ok(master) => master,
        Err(e) => return Err(LabError::InvalidMnemonic(e.to_string())),
    };
    let secp = Secp256k1::new();
    let parsed_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let child = master
        .derive_priv(&secp, &parsed_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    Ok(ExtendedKeyReport {
        derivation_path: path.to_string(),
        xpriv: child.to_string(),
        xpub: Xpub::from_priv(&secp, &child).to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    // todo!("Lab 08: derive one non-hardened public child")
    let parent = Xpub::from_str(parent_xpub).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let secp = Secp256k1::new();
    let child_number =
        ChildNumber::from_normal_idx(index).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    let child = parent
        .derive_pub(&secp, &vec![child_number])
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    Ok(child.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    // todo!("Lab 08: parse the path and inspect its child numbers")
    let parsed =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    Ok(parsed.into_iter().any(|child| child.is_hardened()))
}
