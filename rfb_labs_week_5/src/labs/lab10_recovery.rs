//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::key::XOnlyPublicKey;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};
use std::str::FromStr;

use crate::error::LabError;
use crate::model::{AddressFormat, DerivedAddressSet};
use crate::LabResult;

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic_str: &str,
    passphrase: &str,
    path_str: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = m.to_seed(passphrase);
    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;

    let path =
        DerivationPath::from_str(path_str).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let secp = Secp256k1::new();

    let derived_xprv = master
        .derive_priv(&secp, &path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    let public_key = PublicKey::new(derived_xprv.private_key.public_key(&secp));
    let compressed = CompressedPublicKey::try_from(public_key)
        .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let address = match format {
        AddressFormat::P2pkh => Address::p2pkh(public_key, network),
        AddressFormat::P2sh => Address::p2shwpkh(&compressed, network),
        AddressFormat::P2wpkh => Address::p2wpkh(&compressed, network),
        AddressFormat::P2tr => {
            let xonly = XOnlyPublicKey::from(compressed);
            Address::p2tr(&secp, xonly, None, network)
        }
        AddressFormat::Unknown => {
            return Err(LabError::InvalidAddress(
                "unknown format cannot be derived".into(),
            ))
        }
    };

    Ok(address.to_string())
}

/// Derive index `n` on the BIP44, BIP49, and BIP84 receive branches.
pub fn derive_address_set(
    mnemonic_str: &str,
    passphrase: &str,
    account: u32,
    index: u32,
    network: Network,
) -> LabResult<DerivedAddressSet> {
    let coin_type = if network == Network::Bitcoin { 0 } else { 1 };

    let bip44_path = format!("m/44'/{coin_type}'/{account}'/0/{index}");
    let bip49_path = format!("m/49'/{coin_type}'/{account}'/0/{index}");
    let bip84_path = format!("m/84'/{coin_type}'/{account}'/0/{index}");

    let bip44_p2pkh = derive_address_for_path(
        mnemonic_str,
        passphrase,
        &bip44_path,
        AddressFormat::P2pkh,
        network,
    )?;
    let bip49_p2sh_p2wpkh = derive_address_for_path(
        mnemonic_str,
        passphrase,
        &bip49_path,
        AddressFormat::P2sh,
        network,
    )?;
    let bip84_p2wpkh = derive_address_for_path(
        mnemonic_str,
        passphrase,
        &bip84_path,
        AddressFormat::P2wpkh,
        network,
    )?;

    Ok(DerivedAddressSet {
        bip44_p2pkh,
        bip49_p2sh_p2wpkh,
        bip84_p2wpkh,
    })
}

/// Prove that identical mnemonic, passphrase, path, and network reproduce an address.
pub fn recovery_is_repeatable(
    mnemonic_str: &str,
    passphrase: &str,
    path_str: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let first = derive_address_for_path(mnemonic_str, passphrase, path_str, format, network)?;
    let second = derive_address_for_path(mnemonic_str, passphrase, path_str, format, network)?;
    Ok(first == second)
}

/// Prove that changing only the final index selects a different address.
pub fn changing_index_changes_address(
    mnemonic_str: &str,
    passphrase: &str,
    first_path: &str,
    second_path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let first = derive_address_for_path(mnemonic_str, passphrase, first_path, format, network)?;
    let second = derive_address_for_path(mnemonic_str, passphrase, second_path, format, network)?;
    Ok(first != second)
}
