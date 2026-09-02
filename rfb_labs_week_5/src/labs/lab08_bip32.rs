//! Lab 08 — derive BIP32 extended private and public keys.

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;

use crate::model::ExtendedKeyReport;
use crate::{LabError, LabResult};

fn master_xpriv_key(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<Xpriv> {
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
        .map_err(|err| LabError::InvalidMnemonic(err.to_string()))?;
    let seed = mnemonic.to_seed(passphrase);

    Xpriv::new_master(network, &seed).map_err(|err| LabError::Derivation(err.to_string()))
}

pub(crate) fn parse_derivation_path(path: &str) -> LabResult<DerivationPath> {
    if path != "m" && !path.starts_with("m/") {
        return Err(LabError::InvalidPath(
            "a full derivation path must start with `m`".to_owned(),
        ));
    }

    path.parse::<DerivationPath>()
        .map_err(|err| LabError::InvalidPath(err.to_string()))
}

pub(crate) fn derive_xpriv_at_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<(Xpriv, DerivationPath)> {
    let derivation_path = parse_derivation_path(path)?;
    let master = master_xpriv_key(mnemonic, passphrase, network)?;
    let secp = Secp256k1::new();
    let derived = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|err| LabError::Derivation(err.to_string()))?;

    Ok((derived, derivation_path))
}

fn canonical_path(path: &DerivationPath) -> String {
    if path.as_ref().is_empty() {
        "m".to_owned()
    } else {
        format!("m/{path}")
    }
}

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    Ok(master_xpriv_key(mnemonic, passphrase, network)?.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let (xpriv, derivation_path) = derive_xpriv_at_path(mnemonic, passphrase, path, network)?;
    let secp = Secp256k1::new();
    let xpub = Xpub::from_priv(&secp, &xpriv);

    Ok(ExtendedKeyReport {
        derivation_path: canonical_path(&derivation_path),
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let parent = parent_xpub
        .parse::<Xpub>()
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;
    let child_number = ChildNumber::from_normal_idx(index)
        .map_err(|err| LabError::InvalidPath(err.to_string()))?;
    let secp = Secp256k1::verification_only();
    let child = parent
        .derive_pub(&secp, &child_number)
        .map_err(|err| LabError::Derivation(err.to_string()))?;

    Ok(child.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let path = parse_derivation_path(path)?;
    Ok(path.as_ref().iter().any(ChildNumber::is_hardened))
}
