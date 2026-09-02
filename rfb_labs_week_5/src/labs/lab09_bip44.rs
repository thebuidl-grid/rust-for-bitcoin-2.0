//! Lab 09 — decode BIP44 paths and derive the selected address.

use bitcoin::bip32::{ChildNumber, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::labs::lab08_bip32::{derive_xpriv_at_path, parse_derivation_path};
use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn hardened_index(child: &ChildNumber, level: &str) -> LabResult<u32> {
    match child {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { .. } => Err(LabError::InvalidPath(format!(
            "the {level} level must be hardened"
        ))),
    }
}

fn normal_index(child: &ChildNumber, level: &str) -> LabResult<u32> {
    match child {
        ChildNumber::Normal { index } => Ok(*index),
        ChildNumber::Hardened { .. } => Err(LabError::InvalidPath(format!(
            "the {level} level must not be hardened"
        ))),
    }
}

fn ordinal(index: u32) -> String {
    let number = index + 1;
    let word = match number {
        1 => Some("first"),
        2 => Some("second"),
        3 => Some("third"),
        4 => Some("fourth"),
        5 => Some("fifth"),
        6 => Some("sixth"),
        7 => Some("seventh"),
        8 => Some("eighth"),
        9 => Some("ninth"),
        10 => Some("tenth"),
        _ => None,
    };

    match word {
        Some(word) => word.to_owned(),
        None => {
            let suffix = if (11..=13).contains(&(number % 100)) {
                "th"
            } else {
                match number % 10 {
                    1 => "st",
                    2 => "nd",
                    3 => "rd",
                    _ => "th",
                }
            };
            format!("{number}{suffix}")
        }
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let path = parse_derivation_path(path)?;
    let levels = path.as_ref();
    if levels.len() != 5 {
        return Err(LabError::InvalidPath(
            "a BIP44 path must contain exactly five levels".to_owned(),
        ));
    }

    let purpose = hardened_index(&levels[0], "purpose")?;
    if purpose != 44 {
        return Err(LabError::InvalidPath(
            "a BIP44 path must use hardened purpose 44'".to_owned(),
        ));
    }

    let coin_type = hardened_index(&levels[1], "coin type")?;
    let account = hardened_index(&levels[2], "account")?;
    let change = normal_index(&levels[3], "change")?;
    if change > 1 {
        return Err(LabError::InvalidPath(
            "the BIP44 change level must be 0 (receive) or 1 (change)".to_owned(),
        ));
    }
    let index = normal_index(&levels[4], "address index")?;

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
    let chain = if info.change == 0 {
        "external receive"
    } else {
        "internal change"
    };

    format!(
        "BIP{} purpose, coin type {}, {} account, {} chain, {} address",
        info.purpose,
        info.coin_type,
        ordinal(info.account),
        chain,
        ordinal(info.index),
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    decode_bip44_path(path)?;
    ChildNumber::from_normal_idx(new_index)
        .map_err(|err| LabError::InvalidPath(err.to_string()))?;
    let (prefix, _) = path.rsplit_once('/').ok_or_else(|| {
        LabError::InvalidPath("a BIP44 path must contain an address index".to_owned())
    })?;

    Ok(format!("{prefix}/{new_index}"))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    decode_bip44_path(path)?;
    let (xpriv, _) = derive_xpriv_at_path(mnemonic, passphrase, path, network)?;
    let secp = Secp256k1::new();
    let public_key = Xpub::from_priv(&secp, &xpriv).to_pub();

    Ok(Address::p2pkh(public_key, network).to_string())
}
