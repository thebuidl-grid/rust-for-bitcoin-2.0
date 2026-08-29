//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn expect_hardened(number: &ChildNumber, level: &str) -> LabResult<u32> {
    match number {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { .. } => Err(LabError::InvalidPath(format!("{level} must be hardened"))),
    }
}

fn expect_normal(number: &ChildNumber, level: &str) -> LabResult<u32> {
    match number {
        ChildNumber::Normal { index } => Ok(*index),
        ChildNumber::Hardened { .. } => {
            Err(LabError::InvalidPath(format!("{level} must not be hardened")))
        }
    }
}

fn ordinal(n: u32) -> String {
    match n {
        1 => "first".to_owned(),
        2 => "second".to_owned(),
        3 => "third".to_owned(),
        4 => "fourth".to_owned(),
        5 => "fifth".to_owned(),
        6 => "sixth".to_owned(),
        7 => "seventh".to_owned(),
        8 => "eighth".to_owned(),
        9 => "ninth".to_owned(),
        10 => "tenth".to_owned(),
        other => format!("{other}th"),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(format!("{error}")))?;
    let numbers: Vec<ChildNumber> = derivation_path.into_iter().copied().collect();
    if numbers.len() != 5 {
        return Err(LabError::InvalidPath(format!(
            "expected 5 path levels, found {}",
            numbers.len()
        )));
    }

    Ok(Bip44PathInfo {
        purpose: expect_hardened(&numbers[0], "purpose")?,
        coin_type: expect_hardened(&numbers[1], "coin type")?,
        account: expect_hardened(&numbers[2], "account")?,
        change: expect_normal(&numbers[3], "change")?,
        index: expect_normal(&numbers[4], "index")?,
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let branch = if info.change == 0 { "receive" } else { "change" };
    format!(
        "purpose {}' selects BIP44, coin type {}' selects the coin, account {}' is the {} account (zero-based), the {} branch is index {} ({}), and address index {} is the {} address (zero-based)",
        info.purpose,
        info.coin_type,
        info.account,
        ordinal(info.account + 1),
        branch,
        info.change,
        branch,
        info.index,
        ordinal(info.index + 1),
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
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(format!("{error}")))?;
    let seed = parsed.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master =
        Xpriv::new_master(network, &seed).map_err(|error| LabError::Derivation(format!("{error}")))?;
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(format!("{error}")))?;
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(format!("{error}")))?;
    let public_key = child.to_priv().public_key(&secp);
    Ok(Address::p2pkh(public_key, network).to_string())
}
