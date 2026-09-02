//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};
use std::str::FromStr;

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
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
        .map_err(|error| crate::LabError::InvalidMnemonic(error.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);

    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    let secp = Secp256k1::new();

    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let public_key = PublicKey::new(child.private_key.public_key(&secp));

    let compressed = CompressedPublicKey::try_from(public_key)
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let address = match format {
        AddressFormat::P2pkh => Address::p2pkh(public_key, network),

        AddressFormat::P2sh => Address::p2shwpkh(&compressed, network),

        AddressFormat::P2wpkh => Address::p2wpkh(&compressed, network),

        AddressFormat::P2tr | AddressFormat::Unknown => {
            return Err(crate::LabError::InvalidScript(
                "unsupported address format for Lab 10".to_owned(),
            ));
        }
    };

    Ok(address.to_string())
}

/// Derive index `n` on the BIP44, BIP49, and BIP84 receive branches.
pub fn derive_address_set(
    mnemonic: &str,
    passphrase: &str,
    account: u32,
    index: u32,
    network: Network,
) -> LabResult<DerivedAddressSet> {
    let coin_type = if network == Network::Bitcoin { 0 } else { 1 };

    let bip44_path = format!("m/44'/{}'/{}'/0/{}", coin_type, account, index);
    let bip49_path = format!("m/49'/{}'/{}'/0/{}", coin_type, account, index);
    let bip84_path = format!("m/84'/{}'/{}'/0/{}", coin_type, account, index);

    let bip44_p2pkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip44_path,
        AddressFormat::P2pkh,
        network,
    )?;

    let bip49_p2sh_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip49_path,
        AddressFormat::P2sh,
        network,
    )?;

    let bip84_p2wpkh = derive_address_for_path(
        mnemonic,
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
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let first = derive_address_for_path(mnemonic, passphrase, path, format, network)?;

    let second = derive_address_for_path(mnemonic, passphrase, path, format, network)?;

    Ok(first == second)
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
    let first = derive_address_for_path(mnemonic, passphrase, first_path, format, network)?;

    let second = derive_address_for_path(mnemonic, passphrase, second_path, format, network)?;

    Ok(first != second)
}
