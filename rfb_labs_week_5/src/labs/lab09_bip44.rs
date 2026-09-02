//! Lab 09 — decode BIP44 paths and derive the selected address.

use bitcoin::bip32::ChildNumber;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::labs::lab08_bip32::{derive_xpriv, parse_path};
use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

/// `m / purpose' / coin_type' / account' / change / address_index`.
const BIP44_LEVELS: usize = 5;

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path = parse_path(path)?;
    let levels: &[ChildNumber] = derivation_path.as_ref();

    if levels.len() != BIP44_LEVELS {
        return Err(LabError::InvalidPath(format!(
            "BIP44 needs {BIP44_LEVELS} levels, found {}",
            levels.len()
        )));
    }

    Ok(Bip44PathInfo {
        purpose: hardened_index(levels[0], "purpose")?,
        coin_type: hardened_index(levels[1], "coin type")?,
        account: hardened_index(levels[2], "account")?,
        change: normal_index(levels[3], "change")?,
        index: normal_index(levels[4], "address index")?,
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let Bip44PathInfo {
        purpose,
        coin_type,
        account,
        change,
        index,
    } = *info;
    let branch = if change == 0 { "receive" } else { "change" };

    format!(
        "purpose {purpose}' fixes the BIP44 layout, coin type {coin_type}' fixes the chain, \
         account {account}' is the {} account, chain {change} is the {branch} branch, \
         and index {index} is the {} address on that branch",
        ordinal(account + 1),
        ordinal(index + 1),
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let info = decode_bip44_path(path)?;

    // Everything above the last level is untouched, so the new address stays inside
    // the same account and the same receive/change branch.
    Ok(format!(
        "m/{}'/{}'/{}'/{}/{new_index}",
        info.purpose, info.coin_type, info.account, info.change
    ))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    // Decode first so a malformed or non-BIP44 path fails before any key work.
    decode_bip44_path(path)?;

    let secp = Secp256k1::new();
    let derivation_path = parse_path(path)?;
    let child = derive_xpriv(mnemonic, passphrase, &derivation_path, network)?;
    let public_key = child.to_priv().public_key(&secp);

    Ok(Address::p2pkh(public_key, network).to_string())
}

/// Read a level that BIP44 requires to be hardened.
fn hardened_index(child: ChildNumber, level: &str) -> LabResult<u32> {
    match child {
        ChildNumber::Hardened { index } => Ok(index),
        ChildNumber::Normal { index } => Err(LabError::InvalidPath(format!(
            "the {level} level must be hardened, found {index}"
        ))),
    }
}

/// Read a level that BIP44 requires to be normal so an xpub can cover it.
fn normal_index(child: ChildNumber, level: &str) -> LabResult<u32> {
    match child {
        ChildNumber::Normal { index } => Ok(index),
        ChildNumber::Hardened { index } => Err(LabError::InvalidPath(format!(
            "the {level} level must not be hardened, found {index}'"
        ))),
    }
}

/// Spell a one-based position so a zero-based path level reads naturally.
fn ordinal(position: u32) -> String {
    const WORDS: [&str; 10] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth",
    ];

    match WORDS.get(position.saturating_sub(1) as usize) {
        Some(word) => (*word).to_owned(),
        None => format!("number {position}"),
    }
}
