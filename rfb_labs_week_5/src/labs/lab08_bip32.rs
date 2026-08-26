//! Lab 08 — derive BIP32 extended private and public keys.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;

use crate::model::ExtendedKeyReport;
use crate::{LabError, LabResult};

fn seed(mnemonic: &str, passphrase: &str) -> LabResult<[u8; 64]> {
    let parsed =
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    Ok(parsed.to_seed(passphrase))
}

fn parse_path(path: &str) -> LabResult<DerivationPath> {
    path.parse::<DerivationPath>()
        .map_err(|error| LabError::InvalidPath(error.to_string()))
}

/// Create the master extended private key from a BIP39 recovery setup.
///
/// The seed's chain code (32 extra bytes produced alongside the master private key by
/// HMAC-SHA512) is what makes derivation *deterministic and structured*: it lets every
/// child key be derived reproducibly from the same seed and path, instead of each
/// account needing its own independently-generated, independently-backed-up key.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    let seed = seed(mnemonic, passphrase)?;
    let xpriv = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    Ok(xpriv.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let seed = seed(mnemonic, passphrase)?;
    let derivation_path = parse_path(path)?;
    let secp = Secp256k1::new();

    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let xpub = Xpub::from_priv(&secp, &xpriv);

    Ok(ExtendedKeyReport {
        derivation_path: path.to_owned(),
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
///
/// This is the "watch-only" use case: an xpub can generate every receiving address on
/// its branch without ever holding a private key, so it can sit on a server or in a
/// block explorer with no ability to sign a spend.
pub fn derive_normal_child_xpub(parent_xpub: &str, index: u32) -> LabResult<String> {
    let secp = Secp256k1::new();
    let parent = parent_xpub
        .parse::<Xpub>()
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;
    let child_number = bitcoin::bip32::ChildNumber::from_normal_idx(index)
        .map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let child = parent
        .ckd_pub(&secp, child_number)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    Ok(child.to_string())
}

/// Return whether a textual path contains at least one hardened step.
///
/// Hardened children (`'`/`h`) mix in the *private* key at each step, so they cannot be
/// derived from a parent xpub — only from the corresponding xpriv. That is precisely
/// why account-level and above steps are hardened: it stops a leaked xpub plus a single
/// leaked child private key from letting an attacker walk back up to the parent xpriv.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let derivation_path = parse_path(path)?;
    Ok(derivation_path
        .as_ref()
        .iter()
        .any(|child| child.is_hardened()))
}
