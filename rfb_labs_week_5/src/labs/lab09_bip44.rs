//! Lab 09 — decode BIP44 paths and derive the selected address.

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};
use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PublicKey};
use std::str::FromStr;

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    // todo!("Lab 09: validate and decode all five BIP44 path levels")
    let parsed_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(path.into()));

    let parts = parsed_path
        .as_ref()
        .map_err(|e| LabError::InvalidPath(path.into()))?;

    if parts.len() != 5 {
        return Err(LabError::InvalidPath("BIP44 path needs 5 levels".into()));
    };

    fn index_of(c: &ChildNumber) -> u32 {
        match c {
            ChildNumber::Normal { index } => *index,
            ChildNumber::Hardened { index } => *index,
        }
    }

    let purpose = index_of(&parts[0]);
    let coin_type = index_of(&parts[1]);
    let account = index_of(&parts[2]);
    let change = index_of(&parts[3]);
    let index = index_of(&parts[4]);

    Ok(Bip44PathInfo {
        purpose,
        coin_type,
        account,
        change,
        index,
    })
}

fn ordinal(n: usize) -> String {
    match n {
        1 => "first".into(),
        2 => "second".into(),
        3 => "third".into(),
        4 => "fourth".into(),
        5 => "fifth".into(),
        6 => "sixth".into(),
        7 => "seventh".into(),
        8 => "eighth".into(),
        9 => "ninth".into(),
        10 => "tenth".into(),
        m => format!("{m}th"),
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    // todo!("Lab 09: explain purpose, coin, account, chain, and index")

    let change_label = if info.change == 1 {
        "change"
    } else {
        "receiving"
    };
    format!(
        "Purpose {}; coin type {}; {} account; {} chain; {} address",
        info.purpose,
        info.coin_type,
        ordinal(info.account as usize + 1),
        change_label,
        ordinal(info.index as usize + 1),
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    //  todo!("Lab 09: preserve the branch and replace only the final child")
    let parts: Vec<String> = path.split('/').map(str::to_string).collect();
    Ok(format!(
        "{}/{new_index}",
        parts[..parts.len() - 1].join("/")
    ))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    // todo!("Lab 09: derive the child key and encode its P2PKH address")

    let m =
        Mnemonic::parse(mnemonic).map_err(|_| LabError::InvalidMnemonic(mnemonic.to_string()))?;
    let seed = m.to_seed(passphrase);

    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;

    let secp = Secp256k1::new();
    let parsed_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let child = master
        .derive_priv(&secp, &parsed_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    let public = PublicKey::new(child.private_key.public_key(&secp));
    Ok(Address::p2pkh(&public, network).to_string())
}
