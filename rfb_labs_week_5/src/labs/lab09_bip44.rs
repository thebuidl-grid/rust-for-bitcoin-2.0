//! Lab 09 — decode BIP44 paths and derive the selected address.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::{CompressedPublicKey, Network, PublicKey};
use std::str::FromStr;

use crate::model::Bip44PathInfo;
use crate::LabResult;
use crate::LabError;

/// Parse `m / purpose' / coin_type' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    use bitcoin::bip32::ChildNumber;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let levels: Vec<ChildNumber> = derivation_path.into_iter().copied().collect();

    if levels.len() != 5 {
        return Err(LabError::InvalidPath(
            "BIP44 path must have exactly 5 levels".to_string(),
        ));
    }

    let purpose = match levels[0] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(LabError::InvalidPath("purpose must be hardened".to_string())),
    };

    let coin_type = match levels[1] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(LabError::InvalidPath("coin_type must be hardened".to_string())),
    };

    let account = match levels[2] {
        ChildNumber::Hardened { index } => index,
        _ => return Err(LabError::InvalidPath("account must be hardened".to_string())),
    };

    let change = match levels[3] {
        ChildNumber::Normal { index } => index,
        _ => return Err(LabError::InvalidPath("change must be normal".to_string())),
    };

    let index = match levels[4] {
        ChildNumber::Normal { index } => index,
        _ => return Err(LabError::InvalidPath("index must be normal".to_string())),
    };

    Ok(Bip44PathInfo {
        purpose,
        coin_type,
        account,
        change,
        index,
    })
}

/// Word ordinal helper for small indexes (zero-based input).
fn word_ordinal(n: u32) -> String {
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
        _ => {
            // Fallback to numeric ordinal for larger indexes
            let num = n + 1;
            let suffix = match num % 100 {
                11 | 12 | 13 => "th",
                _ => match num % 10 {
                    1 => "st",
                    2 => "nd",
                    3 => "rd",
                    _ => "th",
                },
            };
            format!("{}{}", num, suffix)
        }
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let chain = if info.change == 0 { "receive" } else { "change" };
    let account_ordinal = word_ordinal(info.account);
    let address_ordinal = word_ordinal(info.index);

    format!(
        "BIP44 path: purpose {purpose}', coin type {coin_type}', {account_ordinal} account (account index {account}), {chain} chain, {address_ordinal} address (index {index})",
        purpose = info.purpose,
        coin_type = info.coin_type,
        account_ordinal = account_ordinal,
        account = info.account,
        chain = chain,
        address_ordinal = address_ordinal,
        index = info.index,
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let info = decode_bip44_path(path)?;
    Ok(format!(
        "m/{}'/{}'/{}' /{}/{}",
        info.purpose, info.coin_type, info.account, info.change, new_index
    ).replace(" ", ""))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let m = Mnemonic::parse_normalized(mnemonic)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = m.to_seed(passphrase);

    let master = Xpriv::new_master(network, &seed)
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    let public_key = PublicKey::new(child.private_key.public_key(&secp));
    let address = bitcoin::Address::p2pkh(public_key, network);

    Ok(address.to_string())
}
