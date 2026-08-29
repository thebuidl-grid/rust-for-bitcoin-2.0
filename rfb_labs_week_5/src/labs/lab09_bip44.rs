//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PublicKey};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

const ORDINALS: [&str; 10] = [
    "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth",
];

fn ordinal(index: u32) -> String {
    match ORDINALS.get(index as usize) {
        Some(word) => (*word).to_owned(),
        None => format!("#{}", index + 1),
    }
}

fn hardened_index(child: ChildNumber) -> LabResult<u32> {
    match child {
        ChildNumber::Hardened { index } => Ok(index),
        ChildNumber::Normal { .. } => Err(LabError::InvalidPath(
            "expected a hardened path level".to_owned(),
        )),
    }
}

fn normal_index(child: ChildNumber) -> LabResult<u32> {
    match child {
        ChildNumber::Normal { index } => Ok(index),
        ChildNumber::Hardened { .. } => Err(LabError::InvalidPath(
            "expected a non-hardened path level".to_owned(),
        )),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let levels = derivation_path.as_ref();
    if levels.len() != 5 {
        return Err(LabError::InvalidPath(format!(
            "expected 5 path levels, found {}",
            levels.len()
        )));
    }

    Ok(Bip44PathInfo {
        purpose: hardened_index(levels[0])?,
        coin_type: hardened_index(levels[1])?,
        account: hardened_index(levels[2])?,
        change: normal_index(levels[3])?,
        index: normal_index(levels[4])?,
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let chain = if info.change == 0 {
        "external/receive"
    } else {
        "internal/change"
    };
    format!(
        "purpose {} on coin type {}, {} account, {} chain, {} address",
        info.purpose,
        info.coin_type,
        ordinal(info.account),
        chain,
        ordinal(info.index)
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let info = decode_bip44_path(path)?;
    Ok(format!(
        "m/{}'/{}'/{}'/{}/{}",
        info.purpose, info.coin_type, info.account, info.change, new_index
    ))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let parsed =
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    let seed = parsed.to_seed(passphrase);
    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let secp = Secp256k1::new();
    let derived = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;

    let public_key = PublicKey::new(derived.private_key.public_key(&secp));
    Ok(Address::p2pkh(public_key, network).to_string())
}
