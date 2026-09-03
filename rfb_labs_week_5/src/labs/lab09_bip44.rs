//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PublicKey};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn seed_from_mnemonic(mnemonic: &str, passphrase: &str) -> LabResult<[u8; 64]> {
    let parsed = Mnemonic::parse_in_normalized(Language::English, mnemonic)
        .map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    Ok(parsed.to_seed_normalized(passphrase))
}

fn ordinal(n: u32) -> String {
    const WORDS: [&str; 10] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth",
    ];
    if let Some(word) = WORDS.get(n as usize) {
        return (*word).to_owned();
    }
    let suffix = match (n + 1) % 10 {
        1 if (n + 1) % 100 != 11 => "st",
        2 if (n + 1) % 100 != 12 => "nd",
        3 if (n + 1) % 100 != 13 => "rd",
        _ => "th",
    };
    format!("{}{}", n + 1, suffix)
}

fn plain_index(child: ChildNumber) -> u32 {
    match child {
        ChildNumber::Normal { index } | ChildNumber::Hardened { index } => index,
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let levels: &[ChildNumber] = derivation_path.as_ref();

    let [purpose, coin_type, account, change, index] = levels else {
        return Err(LabError::InvalidPath(format!(
            "expected 5 BIP44 levels, found {}",
            levels.len()
        )));
    };

    if !purpose.is_hardened() || !coin_type.is_hardened() || !account.is_hardened() {
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
        purpose: plain_index(*purpose),
        coin_type: plain_index(*coin_type),
        account: plain_index(*account),
        change: plain_index(*change),
        index: plain_index(*index),
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
        "purpose {}' selects BIP44, coin type {}' selects the coin, account {} is the {} account (zero-based), \
         the {} branch is used, and index {} is the {} address (zero-based) on that branch.",
        info.purpose,
        info.coin_type,
        info.account,
        ordinal(info.account),
        chain,
        info.index,
        ordinal(info.index)
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    decode_bip44_path(path)?;

    let mut segments: Vec<&str> = path.split('/').collect();
    let new_index_string = new_index.to_string();
    if let Some(last) = segments.last_mut() {
        *last = &new_index_string;
    }
    Ok(segments.join("/"))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let seed = seed_from_mnemonic(mnemonic, passphrase)?;
    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(error.to_string()))?;

    let secp = Secp256k1::new();
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;

    let public = PublicKey::new(child.private_key.public_key(&secp));
    Ok(Address::p2pkh(public, network).to_string())
}
