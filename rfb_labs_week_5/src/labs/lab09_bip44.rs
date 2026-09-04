//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn raw_index(child: ChildNumber) -> u32 {
    match child {
        ChildNumber::Normal { index } | ChildNumber::Hardened { index } => index,
    }
}

fn ordinal_word(zero_based_index: u32) -> String {
    const WORDS: [&str; 10] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth",
    ];
    if let Some(word) = WORDS.get(zero_based_index as usize) {
        return (*word).to_owned();
    }
    let n = zero_based_index + 1;
    let suffix = match (n % 10, n % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let levels: &[ChildNumber] = derivation_path.as_ref();

    let [purpose, coin_type, account, change, index]: [ChildNumber; 5] =
        levels.try_into().map_err(|_| {
            LabError::InvalidPath(format!("expected 5 path levels, found {}", levels.len()))
        })?;

    if !(purpose.is_hardened() && coin_type.is_hardened() && account.is_hardened()) {
        return Err(LabError::InvalidPath(
            "purpose, coin type, and account must be hardened".to_owned(),
        ));
    }
    if change.is_hardened() || index.is_hardened() {
        return Err(LabError::InvalidPath(
            "change and address index must not be hardened".to_owned(),
        ));
    }

    Ok(Bip44PathInfo {
        purpose: raw_index(purpose),
        coin_type: raw_index(coin_type),
        account: raw_index(account),
        change: raw_index(change),
        index: raw_index(index),
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let chain = if info.change == 0 {
        "receive"
    } else {
        "change"
    };
    format!(
        "purpose {p}' selects BIP44, coin type {c}' selects the coin, this is the {a_ord} account \
         (zero-based index {a}), branch {ch} is the {chain} chain, and this is the {i_ord} address \
         (zero-based index {i}) on that chain.",
        p = info.purpose,
        c = info.coin_type,
        a = info.account,
        a_ord = ordinal_word(info.account),
        ch = info.change,
        chain = chain,
        i = info.index,
        i_ord = ordinal_word(info.index),
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
    let parsed =
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    let seed = parsed.to_seed(passphrase);
    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;

    let secp = Secp256k1::new();
    let child_xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let public = child_xpriv.to_priv().public_key(&secp);

    Ok(Address::p2pkh(public, network).to_string())
}
