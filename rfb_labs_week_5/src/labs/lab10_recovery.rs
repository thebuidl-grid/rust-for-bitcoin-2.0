//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, ScriptBuf};

use crate::model::{AddressFormat, DerivedAddressSet};
use crate::{LabError, LabResult};

fn derive_child(mnemonic: &str, passphrase: &str, path: &str, network: Network) -> LabResult<Xpriv> {
    let parsed =
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(format!("{error}")))?;
    let seed = parsed.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master =
        Xpriv::new_master(network, &seed).map_err(|error| LabError::Derivation(format!("{error}")))?;
    let derivation_path =
        DerivationPath::from_str(path).map_err(|error| LabError::InvalidPath(format!("{error}")))?;
    master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(format!("{error}")))
}

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let secp = Secp256k1::new();
    let child = derive_child(mnemonic, passphrase, path, network)?;
    let public_key = child.to_priv().public_key(&secp);

    match format {
        AddressFormat::P2pkh => Ok(Address::p2pkh(public_key, network).to_string()),
        AddressFormat::P2sh => {
            let compressed = CompressedPublicKey::try_from(public_key)
                .map_err(|error| LabError::InvalidKey(format!("{error}")))?;
            let witness_script = ScriptBuf::new_p2wpkh(&compressed.wpubkey_hash());
            let address = Address::p2sh(&witness_script, network)
                .map_err(|error| LabError::InvalidScript(format!("{error}")))?;
            Ok(address.to_string())
        }
        AddressFormat::P2wpkh => {
            let compressed = CompressedPublicKey::try_from(public_key)
                .map_err(|error| LabError::InvalidKey(format!("{error}")))?;
            Ok(Address::p2wpkh(&compressed, network).to_string())
        }
        AddressFormat::P2tr | AddressFormat::Unknown => Err(LabError::InvalidScript(
            "unsupported address format for this lab".to_owned(),
        )),
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
    let coin_type = if network == Network::Bitcoin { 0 } else { 1 };
    let bip44_path = format!("m/44'/{coin_type}'/{account}'/0/{index}");
    let bip49_path = format!("m/49'/{coin_type}'/{account}'/0/{index}");
    let bip84_path = format!("m/84'/{coin_type}'/{account}'/0/{index}");

    Ok(DerivedAddressSet {
        bip44_p2pkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &bip44_path,
            AddressFormat::P2pkh,
            network,
        )?,
        bip49_p2sh_p2wpkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &bip49_path,
            AddressFormat::P2sh,
            network,
        )?,
        bip84_p2wpkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &bip84_path,
            AddressFormat::P2wpkh,
            network,
        )?,
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
