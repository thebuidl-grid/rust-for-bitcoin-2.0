//! Lab 09 — decode BIP44 paths and derive the selected address.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::Network;
use std::str::FromStr;

use crate::model::Bip44PathInfo;
use crate::LabResult;

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| crate::LabError::InvalidPath(e.to_string()))?;

    let path_vec: Vec<_> = derivation_path.into_iter().collect();

    if path_vec.len() != 5 {
        return Err(crate::LabError::InvalidPath(
            "BIP44 path must have exactly 5 levels".to_string(),
        ));
    }

    // Convert ChildNumber to u32, removing hardened bit if present
    let purpose = unharden(u32::from(*path_vec[0]));
    let coin_type = unharden(u32::from(*path_vec[1]));
    let account = unharden(u32::from(*path_vec[2]));
    let change = u32::from(*path_vec[3]);
    let index = u32::from(*path_vec[4]);

    Ok(Bip44PathInfo {
        purpose,
        coin_type,
        account,
        change,
        index,
    })
}

fn unharden(child_index: u32) -> u32 {
    const HARDENED_BIT: u32 = 0x80000000;
    if child_index >= HARDENED_BIT {
        child_index - HARDENED_BIT
    } else {
        child_index
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let coin_name = match info.coin_type {
        0 => "Bitcoin",
        1 => "Testnet",
        _ => "Unknown Coin",
    };

    let chain_name = match info.change {
        0 => "receiving",
        1 => "change",
        _ => "unknown chain",
    };

    let account_ordinal = match info.account {
        0 => "first account".to_string(),
        1 => "second account".to_string(),
        2 => "third account".to_string(),
        n => format!("account {}", n + 1),
    };

    let address_ordinal = match info.index {
        0 => "first address".to_string(),
        1 => "second address".to_string(),
        2 => "third address".to_string(),
        3 => "fourth address".to_string(),
        4 => "fifth address".to_string(),
        5 => "sixth address".to_string(),
        n => format!("address {}", n + 1),
    };

    format!(
        "{} {}, {} {}",
        coin_name, account_ordinal, chain_name, address_ordinal
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() != 6 {
        return Err(crate::LabError::InvalidPath(
            "Invalid BIP44 path structure".to_string(),
        ));
    }

    let new_path = format!(
        "{}/{}/{}/{}/{}/{}",
        parts[0], parts[1], parts[2], parts[3], parts[4], new_index
    );

    Ok(new_path)
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let mnemonic =
        Mnemonic::parse(mnemonic).map_err(|e| crate::LabError::InvalidMnemonic(e.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);
    let master_xpriv = Xpriv::new_master(network, &seed)
        .map_err(|e| crate::LabError::Derivation(e.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| crate::LabError::InvalidPath(e.to_string()))?;

    let derived_xpriv = master_xpriv
        .derive_priv(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path)
        .map_err(|e| crate::LabError::Derivation(e.to_string()))?;

    let pubkey = derived_xpriv
        .to_priv()
        .public_key(&bitcoin::secp256k1::Secp256k1::new());
    let address = bitcoin::Address::p2pkh(&pubkey, network);

    Ok(address.to_string())
}
