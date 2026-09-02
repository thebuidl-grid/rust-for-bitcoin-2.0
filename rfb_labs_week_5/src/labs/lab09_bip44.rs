//! Lab 09 — decode BIP44 paths and derive the selected address.

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PublicKey};
use std::str::FromStr;

use crate::model::Bip44PathInfo;
use crate::LabResult;

fn ordinal_word(index: u32) -> String {
    match index {
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
        _ => format!("number {}", index + 1),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    if !path.starts_with("m/") {
        return Err(crate::LabError::InvalidPath(
            "BIP44 path must start with m/".to_owned(),
        ));
    }

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    let children = derivation_path.as_ref();

    if children.len() != 5 {
        return Err(crate::LabError::InvalidPath(
            "BIP44 path must contain exactly five levels".to_owned(),
        ));
    }

    let purpose = match children[0] {
        ChildNumber::Hardened { index } => index,
        _ => {
            return Err(crate::LabError::InvalidPath(
                "purpose must be hardened".to_owned(),
            ))
        }
    };

    let coin_type = match children[1] {
        ChildNumber::Hardened { index } => index,
        _ => {
            return Err(crate::LabError::InvalidPath(
                "coin type must be hardened".to_owned(),
            ))
        }
    };

    let account = match children[2] {
        ChildNumber::Hardened { index } => index,
        _ => {
            return Err(crate::LabError::InvalidPath(
                "account must be hardened".to_owned(),
            ))
        }
    };

    let change = match children[3] {
        ChildNumber::Normal { index } => index,
        _ => {
            return Err(crate::LabError::InvalidPath(
                "change must be a normal child".to_owned(),
            ))
        }
    };

    let index = match children[4] {
        ChildNumber::Normal { index } => index,
        _ => {
            return Err(crate::LabError::InvalidPath(
                "address index must be a normal child".to_owned(),
            ))
        }
    };

    if purpose != 44 {
        return Err(crate::LabError::InvalidPath(
            "BIP44 purpose must be 44'".to_owned(),
        ));
    }

    if change > 1 {
        return Err(crate::LabError::InvalidPath(
            "BIP44 change level must be 0 or 1".to_owned(),
        ));
    }

    Ok(Bip44PathInfo {
        purpose,
        coin_type,
        account,
        change,
        index,
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let chain = match info.change {
        0 => "receive",
        1 => "change",
        _ => "unknown",
    };

    format!(
        "BIP44 purpose {}, coin type {}, {} account, {} chain, {} address",
        info.purpose,
        info.coin_type,
        ordinal_word(info.account),
        chain,
        ordinal_word(info.index),
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let info = decode_bip44_path(path)?;

    ChildNumber::from_normal_idx(new_index)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

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
    decode_bip44_path(path)?;

    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
        .map_err(|error| crate::LabError::InvalidMnemonic(error.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);

    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    let secp = Secp256k1::new();

    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let public_key = PublicKey::new(child.private_key.public_key(&secp));

    Ok(Address::p2pkh(public_key, network).to_string())
}
