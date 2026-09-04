//! Lab 09 — decode BIP44 paths and derive the selected address.

use bitcoin::bip32::ChildNumber;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::error::LabError;
use crate::labs::common::{derive_xpriv, parse_path};
use crate::model::Bip44PathInfo;
use crate::LabResult;

/// The raw child index, ignoring whether the step is hardened.
fn raw_index(child: &ChildNumber) -> u32 {
    match child {
        ChildNumber::Normal { index } => *index,
        ChildNumber::Hardened { index } => *index,
    }
}

/// Spell out a small ordinal (`1 -> "first"`), falling back to a numeric suffix.
fn ordinal(n: u32) -> String {
    const WORDS: [&str; 21] = [
        "zeroth",
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

    if let Some(word) = WORDS.get(n as usize) {
        return (*word).to_owned();
    }

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
    let parsed = parse_path(path)?;
    let levels: &[ChildNumber] = parsed.as_ref();

    if levels.len() != 5 {
        return Err(LabError::InvalidPath(format!(
            "BIP44 needs exactly 5 levels, found {}",
            levels.len()
        )));
    }

    for (position, level) in levels.iter().enumerate().take(3) {
        if !level.is_hardened() {
            return Err(LabError::InvalidPath(format!(
                "level {position} (purpose/coin/account) must be hardened"
            )));
        }
    }
    for (position, level) in levels.iter().enumerate().skip(3) {
        if level.is_hardened() {
            return Err(LabError::InvalidPath(format!(
                "level {position} (change/index) must not be hardened"
            )));
        }
    }

    Ok(Bip44PathInfo {
        purpose: raw_index(&levels[0]),
        coin_type: raw_index(&levels[1]),
        account: raw_index(&levels[2]),
        change: raw_index(&levels[3]),
        index: raw_index(&levels[4]),
    })
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let coin = match info.coin_type {
        0 => "Bitcoin mainnet (coin type 0)".to_owned(),
        1 => "Bitcoin testnet (coin type 1)".to_owned(),
        other => format!("coin type {other}"),
    };
    let chain = if info.change == 1 {
        "internal change chain"
    } else {
        "external receive chain"
    };

    format!(
        "BIP{} multi-account layout for {coin}, using the {} account (account index {}), \
         the {chain} (chain {}), and the {} address (address index {}).",
        info.purpose,
        ordinal(info.account + 1),
        info.account,
        info.change,
        ordinal(info.index + 1),
        info.index,
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
    let secp = Secp256k1::new();
    let xpriv = derive_xpriv(mnemonic, passphrase, path, network)?;
    let public_key = xpriv.to_priv().public_key(&secp);
    Ok(Address::p2pkh(public_key, network).to_string())
}
