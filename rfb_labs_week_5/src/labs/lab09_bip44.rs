//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn ordinal(n: u32) -> String {
    return match n {
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
        _ => {
            let suffix = match (n % 10, n % 100) {
                (1, 11) | (2, 12) | (3, 13) => "th",
                (1, _) => "st",
                (2, _) => "nd",
                (3, _) => "rd",
                _ => "th",
            };
            format!("{n}{suffix}")
        }
    };
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let levels: Vec<ChildNumber> = derivation_path.as_ref().to_vec();

    if levels.len() != 5 {
        return Err(LabError::InvalidPath(
            "a BIP44 path needs exactly five levels".to_owned(),
        ));
    }

    let hardened_index = |child: ChildNumber| match child {
        ChildNumber::Hardened { index } => Ok(index),
        ChildNumber::Normal { .. } => Err(LabError::InvalidPath(
            "purpose, coin type, and account must be hardened".to_owned(),
        )),
    };
    let normal_index = |child: ChildNumber| match child {
        ChildNumber::Normal { index } => Ok(index),
        ChildNumber::Hardened { .. } => Err(LabError::InvalidPath(
            "change and address index must not be hardened".to_owned(),
        )),
    };

    return Ok(Bip44PathInfo {
        purpose: hardened_index(levels[0])?,
        coin_type: hardened_index(levels[1])?,
        account: hardened_index(levels[2])?,
        change: normal_index(levels[3])?,
        index: normal_index(levels[4])?,
    });
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let chain_name = if info.change == 0 {
        "receiving"
    } else {
        "change"
    };

    return format!(
        "BIP44 path: purpose {purpose}' selects BIP44, coin type {coin_type}' selects the chain, \
the {account_ord} account (account index {account}) holds the funds, the {chain_name} chain \
(change {change}) picks the branch, and the {index_ord} address (address index {index}) is the \
one in use.",
        purpose = info.purpose,
        coin_type = info.coin_type,
        account_ord = ordinal(info.account + 1),
        account = info.account,
        chain_name = chain_name,
        change = info.change,
        index_ord = ordinal(info.index + 1),
        index = info.index,
    );
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let info = decode_bip44_path(path)?;
    return Ok(format!(
        "m/{}'/{}'/{}'/{}/{}",
        info.purpose, info.coin_type, info.account, info.change, new_index
    ));
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let parsed =
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    let seed = parsed.to_seed(passphrase);
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let public = child.to_priv().public_key(&secp);

    return Ok(Address::p2pkh(public, network).to_string());
}
