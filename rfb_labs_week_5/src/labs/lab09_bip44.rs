//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::key::Secp256k1;
use bitcoin::{Address, Network, PublicKey};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() != 6 || parts[0] != "m" {
        return Err(crate::LabError::InvalidPath(
            "BIP44 path must be m/purpose'/coin'/account'/change/index".to_string(),
        ));
    }

    fn hardened(value: &str, name: &str) -> LabResult<u32> {
        let number = value
            .strip_suffix('\'')
            .ok_or_else(|| crate::LabError::InvalidPath(format!("{name} must be hardened")))?;

        number
            .parse::<u32>()
            .map_err(|_| crate::LabError::InvalidPath(format!("invalid {name}: {value}")))
    }

    fn normal(value: &str, name: &str) -> LabResult<u32> {
        if value.ends_with('\'') {
            return Err(crate::LabError::InvalidPath(format!(
                "{name} must not be hardened"
            )));
        }

        value
            .parse::<u32>()
            .map_err(|_| crate::LabError::InvalidPath(format!("invalid {name}: {value}")))
    }

    Ok(Bip44PathInfo {
        purpose: hardened(parts[1], "purpose")?,
        coin_type: hardened(parts[2], "coin type")?,
        account: hardened(parts[3], "account")?,
        change: normal(parts[4], "change")?,
        index: normal(parts[5], "index")?,
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let purpose = match info.purpose {
        44 => "BIP44 legacy P2PKH",
        49 => "BIP49 nested SegWit",
        84 => "BIP84 native SegWit",
        86 => "BIP86 Taproot",
        _ => "unknown address standard",
    };

    let coin = match info.coin_type {
        0 => "Bitcoin mainnet",
        1 => "Bitcoin testnet",
        _ => "an unknown coin",
    };

    let account = match info.account {
        0 => "first",
        1 => "second",
        2 => "third",
        3 => "fourth",
        4 => "fifth",
        5 => "sixth",
        6 => "seventh",
        7 => "eighth",
        8 => "ninth",
        9 => "tenth",
        _ => "other",
    };
    let chain = match info.change {
        0 => "external receiving",
        1 => "internal change",
        _ => "an unknown chain",
    };

    let address = match info.index {
        0 => "first",
        1 => "second",
        2 => "third",
        3 => "fourth",
        4 => "fifth",
        5 => "sixth",
        6 => "seventh",
        7 => "eighth",
        8 => "nineth",
        9 => "third",
        _ => "other",
    };

    format!("{purpose} for {coin}, {account} account, {chain} chain, {address} address",)
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let info = decode_bip44_path(path)?;

    Ok(format!(
        "m/{}'/{}'/{}'/{} /{}",
        info.purpose, info.coin_type, info.account, info.change, new_index
    )
    .replace(" /", "/"))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let info = decode_bip44_path(path)?;

    if info.purpose != 44 {
        return Err(LabError::InvalidPath(format!(
            "BIP44 P2PKH derivation requires purpose 44, got {}",
            info.purpose
        )));
    }

    let master_string = crate::labs::lab08_bip32::master_xpriv(mnemonic, passphrase, network)?;

    let master = Xpriv::from_str(&master_string)
        .map_err(|error| LabError::Derivation(format!("failed to parse master xpriv: {error}")))?;

    let derivation_path = DerivationPath::from_str(path).map_err(|error| {
        LabError::InvalidPath(format!("invalid derivation path `{path}`: {error}"))
    })?;

    let secp = Secp256k1::new();

    let derived = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| {
            LabError::Derivation(format!("failed to derive BIP44 address key: {error}"))
        })?;

    let public_key = PublicKey::new(derived.private_key.public_key(&secp));
    let address = Address::p2pkh(&public_key, network);

    Ok(address.to_string())
}
