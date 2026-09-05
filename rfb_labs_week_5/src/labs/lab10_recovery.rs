//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use std::str::FromStr;

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};

use crate::model::{AddressFormat, DerivedAddressSet};
use crate::{LabError, LabResult};

fn seed_from_mnemonic(mnemonic: &str, passphrase: &str) -> LabResult<[u8; 64]> {
    let parsed = Mnemonic::parse_in_normalized(Language::English, mnemonic)
        .map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    Ok(parsed.to_seed_normalized(passphrase))
}

fn derive_child_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<(PublicKey, CompressedPublicKey)> {
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
    let compressed = CompressedPublicKey::try_from(public)
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;
    Ok((public, compressed))
}

fn coin_type(network: Network) -> u32 {
    match network {
        Network::Bitcoin => 0,
        _ => 1,
    }
}

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let (public, compressed) = derive_child_keys(mnemonic, passphrase, path, network)?;

    let address = match format {
        AddressFormat::P2pkh => Address::p2pkh(public, network),
        AddressFormat::P2sh => Address::p2shwpkh(&compressed, network),
        AddressFormat::P2wpkh => Address::p2wpkh(&compressed, network),
        AddressFormat::P2tr | AddressFormat::Unknown => {
            return Err(LabError::InvalidScript(
                "Lab 10 only derives P2PKH, P2SH-wrapped P2WPKH, and native P2WPKH addresses"
                    .to_owned(),
            ))
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
    let coin = coin_type(network);
    let bip44_path = format!("m/44'/{coin}'/{account}'/0/{index}");
    let bip49_path = format!("m/49'/{coin}'/{account}'/0/{index}");
    let bip84_path = format!("m/84'/{coin}'/{account}'/0/{index}");

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
