//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bitcoin::Network;

use crate::model::{AddressFormat, DerivedAddressSet};
use crate::LabResult;

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let report = crate::labs::lab08_bip32::derive_extended_keys(mnemonic, passphrase, path, network)?;

    use std::str::FromStr;
    let xpub = bitcoin::bip32::Xpub::from_str(&report.xpub)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    let pubkey = bitcoin::PublicKey::new(xpub.public_key);

    match format {
        AddressFormat::P2pkh => {
            crate::labs::lab02_p2pkh::derive_p2pkh_address(&pubkey.to_string(), network)
        }
        AddressFormat::P2sh => {
            let compressed = bitcoin::CompressedPublicKey::try_from(pubkey)
                .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
            let address = bitcoin::Address::p2shwpkh(&compressed, network);
            Ok(address.to_string())
        }
        AddressFormat::P2wpkh => {
            crate::labs::lab04_p2wpkh::derive_p2wpkh_address(&pubkey.to_string(), network)
        }
        _ => Err(crate::error::LabError::Derivation("Unsupported address format".to_owned())),
    }
}

/// Derive index `n` on the BIP44, BIP49, and BIP84 receive branches.
pub fn derive_address_set(
    mnemonic: &str,
    passphrase: &str,
    account: u32,
    index: u32,
    network: Network,
) -> LabResult<DerivedAddressSet> {
    let coin_type = match network {
        bitcoin::Network::Bitcoin => 0,
        _ => 1,
    };

    let bip44_path = format!("m/44'/{}'/{}'/0/{}", coin_type, account, index);
    let bip49_path = format!("m/49'/{}'/{}'/0/{}", coin_type, account, index);
    let bip84_path = format!("m/84'/{}'/{}'/0/{}", coin_type, account, index);

    let bip44_p2pkh = derive_address_for_path(mnemonic, passphrase, &bip44_path, AddressFormat::P2pkh, network)?;
    let bip49_p2sh_p2wpkh = derive_address_for_path(mnemonic, passphrase, &bip49_path, AddressFormat::P2sh, network)?;
    let bip84_p2wpkh = derive_address_for_path(mnemonic, passphrase, &bip84_path, AddressFormat::P2wpkh, network)?;

    Ok(DerivedAddressSet {
        bip44_p2pkh,
        bip49_p2sh_p2wpkh,
        bip84_p2wpkh,
    })
}

/// Prove that identical mnemonic, passphrase, path, and network reproduce an address.
pub fn recovery_is_repeatable(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let addr1 = derive_address_for_path(mnemonic, passphrase, path, format, network)?;
    let addr2 = derive_address_for_path(mnemonic, passphrase, path, format, network)?;
    Ok(addr1 == addr2)
}

/// Prove that changing only the final index selects a different address.
pub fn changing_index_changes_address(
    mnemonic: &str,
    passphrase: &str,
    first_path: &str,
    second_path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let addr1 = derive_address_for_path(mnemonic, passphrase, first_path, format, network)?;
    let addr2 = derive_address_for_path(mnemonic, passphrase, second_path, format, network)?;
    Ok(addr1 != addr2)
}

