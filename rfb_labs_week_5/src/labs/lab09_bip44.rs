//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn master_key(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<Xpriv> {
    let mnemonic =
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    let seed = mnemonic.to_seed(passphrase);
    Xpriv::new_master(network, &seed).map_err(|error| LabError::Derivation(error.to_string()))
}

fn raw_index(child: &ChildNumber) -> u32 {
    match *child {
        ChildNumber::Normal { index } | ChildNumber::Hardened { index } => index,
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
        _ => format!("{n}th"),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let components: Vec<ChildNumber> = derivation_path.into();

    let [purpose, coin_type, account, change, index] = <[ChildNumber; 5]>::try_from(components)
        .map_err(|_| LabError::InvalidPath("a BIP44 path needs exactly five levels".to_owned()))?;

    let all_hardened = purpose.is_hardened() && coin_type.is_hardened() && account.is_hardened();
    let all_normal = change.is_normal() && index.is_normal();
    if !all_hardened || !all_normal {
        return Err(LabError::InvalidPath(
            "purpose, coin type, and account must be hardened; change and index must not"
                .to_owned(),
        ));
    }

    Ok(Bip44PathInfo {
        purpose: raw_index(&purpose),
        coin_type: raw_index(&coin_type),
        account: raw_index(&account),
        change: raw_index(&change),
        index: raw_index(&index),
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let account_ordinal = ordinal(info.account + 1);
    let address_ordinal = ordinal(info.index + 1);
    let chain_name = if info.change == 0 {
        "external/receiving"
    } else {
        "internal/change"
    };

    format!(
        "Purpose {purpose}' selects BIP44, coin type {coin_type}' selects the network's \
         registered coin, account {account}' is the {account_ordinal} account (accounts are \
         zero-based), chain {change} is the {chain_name} chain, and index {index} selects the \
         {address_ordinal} address on that chain.",
        purpose = info.purpose,
        coin_type = info.coin_type,
        account = info.account,
        change = info.change,
        index = info.index,
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    // Validate the path is well-formed before doing textual surgery on it.
    DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;

    let mut parts: Vec<String> = path.split('/').map(str::to_owned).collect();
    match parts.last_mut() {
        Some(last) => *last = new_index.to_string(),
        None => {
            return Err(LabError::InvalidPath(
                "path has no address index".to_owned(),
            ))
        }
    }

    Ok(parts.join("/"))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let secp = Secp256k1::new();
    let master = master_key(mnemonic, passphrase, network)?;
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;

    let xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let public_key = xpriv.to_priv().public_key(&secp);

    Ok(Address::p2pkh(public_key, network).to_string())
}
