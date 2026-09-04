//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, DerivationPath};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use super::lab08_bip32::derive_priv_at_path;
use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn normal_index(child: &ChildNumber) -> LabResult<u32> {
    match child {
        ChildNumber::Normal { index } => Ok(*index),
        ChildNumber::Hardened { .. } => Err(LabError::InvalidPath(
            "expected a non-hardened path step".to_owned(),
        )),
    }
}

fn hardened_index(child: &ChildNumber) -> LabResult<u32> {
    match child {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { .. } => Err(LabError::InvalidPath(
            "expected a hardened path step".to_owned(),
        )),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let parsed =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let steps: Vec<&ChildNumber> = (&parsed).into_iter().collect();

    if steps.len() != 5 {
        return Err(LabError::InvalidPath(
            "a BIP44 path needs purpose, coin, account, change, and index".to_owned(),
        ));
    }

    Ok(Bip44PathInfo {
        purpose: hardened_index(steps[0])?,
        coin_type: hardened_index(steps[1])?,
        account: hardened_index(steps[2])?,
        change: normal_index(steps[3])?,
        index: normal_index(steps[4])?,
    })
}

fn ordinal(n: u32) -> String {
    const WORDS: [&str; 11] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth", "eleventh",
    ];
    match WORDS.get(n as usize) {
        Some(word) => (*word).to_owned(),
        None => {
            let suffix = match n % 10 {
                1 if n % 100 != 11 => "st",
                2 if n % 100 != 12 => "nd",
                3 if n % 100 != 13 => "rd",
                _ => "th",
            };
            format!("{}{suffix}", n + 1)
        }
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let chain = if info.change == 0 {
        "receive (external) change chain"
    } else {
        "change (internal) chain"
    };

    format!(
        "purpose {}' selects BIP44, coin' {} selects the coin, this is the {} account, \
         using the {chain}, and the {} address in that chain",
        info.purpose,
        info.coin_type,
        ordinal(info.account),
        ordinal(info.index),
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
    let xpriv = derive_priv_at_path(mnemonic, passphrase, path, network)?;
    let secp = Secp256k1::new();
    let public = xpriv.to_priv().public_key(&secp);
    Ok(Address::p2pkh(public, network).to_string())
}
