//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn hardened_index(child: ChildNumber, level: &str) -> LabResult<u32> {
    match child {
        ChildNumber::Hardened { index } => Ok(index),
        ChildNumber::Normal { .. } => Err(LabError::InvalidPath(format!(
            "{level} must be a hardened step"
        ))),
    }
}

fn normal_index(child: ChildNumber, level: &str) -> LabResult<u32> {
    match child {
        ChildNumber::Normal { index } => Ok(index),
        ChildNumber::Hardened { .. } => Err(LabError::InvalidPath(format!(
            "{level} must not be hardened"
        ))),
    }
}

fn ordinal(n: u32) -> String {
    match n {
        0 => "first".to_string(),
        1 => "second".to_string(),
        2 => "third".to_string(),
        3 => "fourth".to_string(),
        4 => "fifth".to_string(),
        5 => "sixth".to_string(),
        6 => "seventh".to_string(),
        7 => "eighth".to_string(),
        8 => "ninth".to_string(),
        9 => "tenth".to_string(),
        other => format!("index {other}"),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let steps = derivation_path.as_ref();

    if steps.len() != 5 {
        return Err(LabError::InvalidPath(format!(
            "expected 5 BIP44 levels, found {}",
            steps.len()
        )));
    }

    Ok(Bip44PathInfo {
        purpose: hardened_index(steps[0], "purpose")?,
        coin_type: hardened_index(steps[1], "coin type")?,
        account: hardened_index(steps[2], "account")?,
        change: normal_index(steps[3], "change")?,
        index: normal_index(steps[4], "index")?,
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
        "purpose {p}' selects BIP44, coin type {c}' selects the coin, account {a}' is the {a_ord} \
         account, change level {ch} is the {branch} branch, and address index {i} is the {i_ord} address.",
        p = info.purpose,
        c = info.coin_type,
        a = info.account,
        a_ord = ordinal(info.account),
        ch = info.change,
        branch = branch,
        i = info.index,
        i_ord = ordinal(info.index),
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
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let public = child.to_priv().public_key(&secp);

    Ok(Address::p2pkh(public, network).to_string())
}
