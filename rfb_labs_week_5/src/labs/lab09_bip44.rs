//! Lab 09 — decode BIP44 paths and derive the selected address.

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::key::PublicKey;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network};
use std::str::FromStr;

use crate::error::LabError;
use crate::model::Bip44PathInfo;
use crate::LabResult;

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() != 6 || parts[0] != "m" {
        return Err(LabError::InvalidPath(path.to_owned()));
    }
    let mut values = Vec::new();
    for part in &parts[1..4] {
        if !part.ends_with('\'') {
            return Err(LabError::InvalidPath(path.to_owned()));
        }
        values.push(
            part[..part.len() - 1]
                .parse::<u32>()
                .map_err(|_| LabError::InvalidPath(path.to_owned()))?,
        );
    }
    let change = parts[4]
        .parse::<u32>()
        .map_err(|_| LabError::InvalidPath(path.to_owned()))?;
    let index = parts[5]
        .parse::<u32>()
        .map_err(|_| LabError::InvalidPath(path.to_owned()))?;
    Ok(Bip44PathInfo {
        purpose: values[0],
        coin_type: values[1],
        account: values[2],
        change,
        index,
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    format!(
        "purpose {} uses coin type {}; third account is {} account, {} chain, and {} address",
        info.purpose,
        info.coin_type,
        ordinal(info.account),
        if info.change == 0 {
            "receive"
        } else {
            "change"
        },
        ordinal_words(info.index),
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
    let seed_mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, mnemonic)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let master = Xpriv::new_master(network, &seed_mnemonic.to_seed(passphrase))
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let secp = Secp256k1::new();
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    let public = PublicKey::new(child.private_key.public_key(&secp));
    let compressed =
        CompressedPublicKey::try_from(public).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    Ok(Address::p2pkh(&compressed, network).to_string())
}

fn ordinal(value: u32) -> String {
    let suffix = match value % 100 {
        11..=13 => "th",
        _ => match value % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{}{}", value + 1, suffix)
}

fn ordinal_words(value: u32) -> String {
    match value + 1 {
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
        number => ordinal(number - 1),
    }
}
