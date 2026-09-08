//! Lab 09 — decode BIP44 paths and derive the selected address.

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PublicKey};
use std::str::FromStr;

use crate::error::LabError;
use crate::model::Bip44PathInfo;
use crate::LabResult;

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path_str: &str) -> LabResult<Bip44PathInfo> {
    let path =
        DerivationPath::from_str(path_str).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let children: Vec<ChildNumber> = path.into_iter().copied().collect();

    if children.len() != 5 {
        return Err(LabError::InvalidPath(
            "BIP44 path must contain exactly 5 levels".into(),
        ));
    }

    let purpose = match children[0] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(LabError::InvalidPath("purpose must be hardened".into())),
    };

    let coin_type = match children[1] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(LabError::InvalidPath("coin_type must be hardened".into())),
    };

    let account = match children[2] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(LabError::InvalidPath("account must be hardened".into())),
    };

    let change = match children[3] {
        ChildNumber::Normal { index } => index,
        _ => return Err(LabError::InvalidPath("change must be normal".into())),
    };

    let index = match children[4] {
        ChildNumber::Normal { index } => index,
        _ => return Err(LabError::InvalidPath("index must be normal".into())),
    };

    Ok(Bip44PathInfo {
        purpose,
        coin_type,
        account,
        change,
        index,
    })
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
        k => format!("{}th", k + 1),
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let acc_str = format!("{} account", ordinal(info.account));
    let chain_str = if info.change == 1 {
        "internal (change) branch"
    } else {
        "external (receive) branch"
    };
    let addr_str = format!("{} address", ordinal(info.index));

    format!(
        "BIP{} path for coin {}, {}, {}, {}",
        info.purpose, info.coin_type, acc_str, chain_str, addr_str
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path_str: &str, new_index: u32) -> LabResult<String> {
    let _info = decode_bip44_path(path_str)?;
    let mut parts: Vec<&str> = path_str.split('/').collect();
    if parts.len() != 6 {
        return Err(LabError::InvalidPath("invalid path structure".into()));
    }
    let new_idx_str = new_index.to_string();
    parts[5] = &new_idx_str;
    Ok(parts.join("/"))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic_str: &str,
    passphrase: &str,
    path_str: &str,
    network: Network,
) -> LabResult<String> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = m.to_seed(passphrase);
    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;

    let path =
        DerivationPath::from_str(path_str).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let secp = Secp256k1::new();

    let derived_xprv = master
        .derive_priv(&secp, &path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    let public_key = PublicKey::new(derived_xprv.private_key.public_key(&secp));
    let address = Address::p2pkh(public_key, network);
    Ok(address.to_string())
}
