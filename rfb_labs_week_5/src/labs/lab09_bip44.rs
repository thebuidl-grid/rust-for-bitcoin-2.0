//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PublicKey};

use crate::error::LabError;
use crate::model::Bip44PathInfo;
use crate::LabResult;

fn ordinal(n: u32) -> String {
    match n {
        0 => "first".to_owned(),
        1 => "second".to_owned(),
        2 => "third".to_owned(),
        3 => "fourth".to_owned(),
        4 => "fifth".to_owned(),
        5 => "sixth".to_owned(),
        6 => "seventh".to_owned(),
        7 => "eighth".to_owned(),
        8 => "ninth".to_owned(),
        9 => "tenth".to_owned(),
        _ => format!("{}th", n + 1),
    }
}

fn hardened_value(child: ChildNumber) -> LabResult<u32> {
    match child {
        ChildNumber::Hardened { index } => Ok(index),
        ChildNumber::Normal { .. } => Err(LabError::InvalidPath(
            "expected a hardened path element".to_owned(),
        )),
    }
}

fn normal_value(child: ChildNumber) -> LabResult<u32> {
    match child {
        ChildNumber::Normal { index } => Ok(index),
        ChildNumber::Hardened { .. } => Err(LabError::InvalidPath(
            "expected a normal path element".to_owned(),
        )),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let children: Vec<ChildNumber> = derivation_path.into_iter().copied().collect();

    if children.len() != 5 {
        return Err(LabError::InvalidPath(format!(
            "BIP44 paths need exactly 5 levels, got {}",
            children.len()
        )));
    }

    Ok(Bip44PathInfo {
        purpose: hardened_value(children[0])?,
        coin_type: hardened_value(children[1])?,
        account: hardened_value(children[2])?,
        change: normal_value(children[3])?,
        index: normal_value(children[4])?,
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let branch = if info.change == 0 {
        "receive"
    } else {
        "change"
    };

    format!(
        "purpose {} (BIP44), coin type {}, the {} account, the {} branch, the {} address",
        info.purpose,
        info.coin_type,
        ordinal(info.account),
        branch,
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
    let parsed_mnemonic = mnemonic
        .parse::<Mnemonic>()
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = parsed_mnemonic.to_seed(passphrase);

    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let secp = Secp256k1::new();
    let child_xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    let public = PublicKey::new(child_xpriv.private_key.public_key(&secp));
    Ok(Address::p2pkh(public, network).to_string())
}
