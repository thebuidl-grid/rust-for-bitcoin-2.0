//! Lab 09 — decode BIP44 paths and derive the selected address.

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path = path
        .parse::<DerivationPath>()
        .map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let steps: Vec<ChildNumber> = derivation_path.into();

    let invalid = || LabError::InvalidPath(format!("expected 5 BIP44 levels, got {path}"));

    if steps.len() != 5 {
        return Err(invalid());
    }

    let hardened = |step: ChildNumber| match step {
        ChildNumber::Hardened { index } => Ok(index),
        ChildNumber::Normal { .. } => Err(invalid()),
    };
    let normal = |step: ChildNumber| match step {
        ChildNumber::Normal { index } => Ok(index),
        ChildNumber::Hardened { .. } => Err(invalid()),
    };

    Ok(Bip44PathInfo {
        purpose: hardened(steps[0])?,
        coin_type: hardened(steps[1])?,
        account: hardened(steps[2])?,
        change: normal(steps[3])?,
        index: normal(steps[4])?,
    })
}

fn ordinal(one_based: u32) -> String {
    const WORDS: [&str; 10] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth",
    ];
    match WORDS.get((one_based.saturating_sub(1)) as usize) {
        Some(word) => (*word).to_owned(),
        None => match one_based % 10 {
            1 if one_based % 100 != 11 => format!("{one_based}st"),
            2 if one_based % 100 != 12 => format!("{one_based}nd"),
            3 if one_based % 100 != 13 => format!("{one_based}rd"),
            _ => format!("{one_based}th"),
        },
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let branch = if info.change == 0 {
        "receive"
    } else {
        "change"
    };

    format!(
        "BIP{} account-structured path: coin type {}, {} account, {} branch, {} address.",
        info.purpose,
        info.coin_type,
        ordinal(info.account + 1),
        branch,
        ordinal(info.index + 1),
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let derivation_path = path
        .parse::<DerivationPath>()
        .map_err(|error| LabError::InvalidPath(error.to_string()))?;
    let mut steps: Vec<ChildNumber> = derivation_path.into();

    let last = steps
        .last_mut()
        .ok_or_else(|| LabError::InvalidPath("path has no address index".to_owned()))?;
    if !last.is_normal() {
        return Err(LabError::InvalidPath(
            "final BIP44 level must be a normal (non-hardened) index".to_owned(),
        ));
    }
    *last = ChildNumber::from_normal_idx(new_index)
        .map_err(|error| LabError::InvalidPath(error.to_string()))?;

    let rebuilt: DerivationPath = steps.into();
    Ok(format!("m/{rebuilt}"))
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
    let derivation_path = path
        .parse::<DerivationPath>()
        .map_err(|error| LabError::InvalidPath(error.to_string()))?;

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;

    let private_key = child.to_priv();
    let public_key = private_key.public_key(&secp);

    Ok(Address::p2pkh(public_key, network).to_string())
}
