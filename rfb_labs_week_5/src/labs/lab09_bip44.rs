//! Lab 09 — decode BIP44 paths and derive the selected address.

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::{Address, Network, PublicKey};
use std::str::FromStr;

use crate::model::Bip44PathInfo;
use crate::LabResult;

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() != 6 || parts[0] != "m" {
        return Err(crate::LabError::InvalidPath(
            "BIP44 path must contain exactly five levels after m".to_string(),
        ));
    }

    let parse_hardened = |value: &str, name: &str| -> LabResult<u32> {
        if !value.ends_with('\'') {
            return Err(crate::LabError::InvalidPath(format!(
                "{name} must be hardened"
            )));
        }

        value[..value.len() - 1]
            .parse::<u32>()
            .map_err(|error| crate::LabError::InvalidPath(error.to_string()))
    };

    let purpose = parse_hardened(parts[1], "purpose")?;
    let coin_type = parse_hardened(parts[2], "coin type")?;
    let account = parse_hardened(parts[3], "account")?;

    let change = parts[4]
        .parse::<u32>()
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    let index = parts[5]
        .parse::<u32>()
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

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
    let account_name = match info.account {
        0 => "first",
        1 => "second",
        2 => "third",
        n => {
            return format!(
                "purpose {} for coin type {}, {} account, {} branch, {} address",
                info.purpose,
                info.coin_type,
                ordinal(n + 1),
                if info.change == 0 {
                    "receive"
                } else {
                    "change"
                },
                ordinal(info.index + 1),
            )
        }
    };

    let branch = if info.change == 0 {
        "receive"
    } else {
        "change"
    };

    format!(
        "purpose {} for coin type {}, {} account, {} branch, {} address",
        info.purpose,
        info.coin_type,
        account_name,
        branch,
        ordinal(info.index + 1),
    )
}

fn ordinal(number: u32) -> String {
    match number {
        1 => "first".to_string(),
        2 => "second".to_string(),
        3 => "third".to_string(),
        4 => "fourth".to_string(),
        5 => "fifth".to_string(),
        6 => "sixth".to_string(),
        7 => "seventh".to_string(),
        8 => "eighth".to_string(),
        9 => "ninth".to_string(),
        10 => "tenth".to_string(),
        n => format!("{n}th"),
    }
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
    let info = decode_bip44_path(path)?;

    let seed_hex = crate::labs::lab07_bip39::mnemonic_seed_hex(mnemonic, passphrase)?;

    let seed =
        hex::decode(seed_hex).map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let secp = bitcoin::secp256k1::Secp256k1::new();

    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let public_key = PublicKey::new(child.private_key.public_key(&secp));

    let address = Address::p2pkh(public_key, network);

    let _ = info;

    Ok(address.to_string())
}
