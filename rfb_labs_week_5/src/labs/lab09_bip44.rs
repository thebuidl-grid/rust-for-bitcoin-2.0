//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PublicKey};

use crate::labs::util::seed_bytes;
use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

const ORDINAL_WORDS: [&str; 20] = [
    "first",
    "second",
    "third",
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "eighth",
    "ninth",
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
    "fourteenth",
    "fifteenth",
    "sixteenth",
    "seventeenth",
    "eighteenth",
    "nineteenth",
    "twentieth",
];

fn ordinal_word(one_based: u32) -> String {
    match ORDINAL_WORDS.get((one_based.saturating_sub(1)) as usize) {
        Some(word) => word.to_string(),
        None => format!("{one_based}th"),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let children = derivation_path.as_ref();

    if children.len() != 5 {
        return Err(LabError::InvalidPath(format!(
            "a BIP44 path needs 5 levels, found {}",
            children.len()
        )));
    }

    let [purpose, coin_type, account, change, index] = [
        children[0],
        children[1],
        children[2],
        children[3],
        children[4],
    ];

    if !purpose.is_hardened() || !coin_type.is_hardened() || !account.is_hardened() {
        return Err(LabError::InvalidPath(
            "purpose, coin type, and account must be hardened".to_string(),
        ));
    }
    if change.is_hardened() || index.is_hardened() {
        return Err(LabError::InvalidPath(
            "change and address index must not be hardened".to_string(),
        ));
    }

    Ok(Bip44PathInfo {
        purpose: plain_index(purpose),
        coin_type: plain_index(coin_type),
        account: plain_index(account),
        change: plain_index(change),
        index: plain_index(index),
    })
}

/// Extract a child number's plain index, without the hardened marker bit that
/// `u32::from(ChildNumber)` folds into hardened values.
fn plain_index(child: ChildNumber) -> u32 {
    match child {
        ChildNumber::Normal { index } => index,
        ChildNumber::Hardened { index } => index,
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let account_word = ordinal_word(info.account + 1);
    let index_word = ordinal_word(info.index + 1);
    let branch = if info.change == 0 {
        "receive"
    } else {
        "change"
    };

    format!(
        "purpose {purpose}' selects BIP44, coin type {coin_type}' selects the coin, account \
         {account}' is the {account_word} account, chain {change} is the {branch} branch, and \
         index {index} is the {index_word} address.",
        purpose = info.purpose,
        coin_type = info.coin_type,
        account = info.account,
        account_word = account_word,
        change = info.change,
        branch = branch,
        index = info.index,
        index_word = index_word,
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
    let seed = seed_bytes(mnemonic, passphrase)?;
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let public = PublicKey::new(child.private_key.public_key(&secp));

    Ok(Address::p2pkh(public, network).to_string())
}
