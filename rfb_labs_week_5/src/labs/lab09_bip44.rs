//! Lab 09 — decode BIP44 paths and derive the selected address.

use bitcoin::Network;

use crate::model::Bip44PathInfo;
use crate::LabResult;

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    use std::str::FromStr;
    use bitcoin::bip32::{DerivationPath, ChildNumber};

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| crate::error::LabError::InvalidPath(e.to_string()))?;

    let path_vec: Vec<ChildNumber> = derivation_path.into();
    if path_vec.len() != 5 {
        return Err(crate::error::LabError::InvalidPath(
            "BIP44 path must have exactly 5 levels".to_owned(),
        ));
    }

    let purpose = match path_vec[0] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(crate::error::LabError::InvalidPath("Purpose must be hardened".to_owned())),
    };

    let coin_type = match path_vec[1] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(crate::error::LabError::InvalidPath("Coin type must be hardened".to_owned())),
    };

    let account = match path_vec[2] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(crate::error::LabError::InvalidPath("Account must be hardened".to_owned())),
    };

    let change = match path_vec[3] {
        ChildNumber::Normal { index } => index,
        _ => return Err(crate::error::LabError::InvalidPath("Change must be non-hardened".to_owned())),
    };

    let index = match path_vec[4] {
        ChildNumber::Normal { index } => index,
        _ => return Err(crate::error::LabError::InvalidPath("Address index must be non-hardened".to_owned())),
    };

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
    let account_ordinal = match info.account {
        0 => "first".to_owned(),
        1 => "second".to_owned(),
        2 => "third".to_owned(),
        n => format!("{}th", n + 1),
    };
    let address_ordinal = match info.index {
        0 => "first".to_owned(),
        1 => "second".to_owned(),
        2 => "third".to_owned(),
        3 => "fourth".to_owned(),
        4 => "fifth".to_owned(),
        5 => "sixth".to_owned(),
        n => format!("{}th", n + 1),
    };
    let chain_desc = match info.change {
        0 => "receive",
        1 => "change",
        _ => "unknown",
    };
    let coin_desc = match info.coin_type {
        0 => "Bitcoin mainnet",
        1 => "Bitcoin testnet",
        _ => "other coin",
    };
    format!(
        "BIP44 path: purpose {}, coin {}, {} account, {}, {} address",
        info.purpose, coin_desc, account_ordinal, chain_desc, address_ordinal
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    use std::str::FromStr;
    let derivation_path = bitcoin::bip32::DerivationPath::from_str(path)
        .map_err(|e| crate::error::LabError::InvalidPath(e.to_string()))?;
    let mut path_vec: Vec<bitcoin::bip32::ChildNumber> = derivation_path.into();
    if path_vec.is_empty() {
        return Err(crate::error::LabError::InvalidPath("Path cannot be empty".to_owned()));
    }
    let len = path_vec.len();
    path_vec[len - 1] = bitcoin::bip32::ChildNumber::Normal { index: new_index };
    let new_path = bitcoin::bip32::DerivationPath::from(path_vec);
    let mut new_path_str = new_path.to_string();
    if path.starts_with("m/") && !new_path_str.starts_with("m/") {
        new_path_str = format!("m/{}", new_path_str);
    }
    Ok(new_path_str)
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let _ = decode_bip44_path(path)?;
    let report = crate::labs::lab08_bip32::derive_extended_keys(mnemonic, passphrase, path, network)?;

    use std::str::FromStr;
    let xpub = bitcoin::bip32::Xpub::from_str(&report.xpub)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    let pubkey = bitcoin::PublicKey::new(xpub.public_key);

    crate::labs::lab02_p2pkh::derive_p2pkh_address(&pubkey.to_string(), network)
}

