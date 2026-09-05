//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::{Address, CompressedPublicKey, Network};
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
    let seed_hex = crate::labs::lab07_bip39::mnemonic_seed_hex(mnemonic, passphrase)?;

    let seed =
        hex::decode(seed_hex).map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|error| crate::LabError::InvalidPath(error.to_string()))?;

    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let secp = bitcoin::secp256k1::Secp256k1::new();

    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| crate::LabError::Derivation(error.to_string()))?;

    let public_key = CompressedPublicKey(child.private_key.public_key(&secp));

    let address = match format {
        AddressFormat::P2pkh => Address::p2pkh(public_key, network),
        AddressFormat::P2sh => Address::p2shwpkh(&public_key, network),
        AddressFormat::P2wpkh => Address::p2wpkh(&public_key, network),
        AddressFormat::P2tr => {
            return Err(crate::LabError::Derivation(
                "P2TR is not required for this lab".to_string(),
            ));
        }
        AddressFormat::Unknown => {
            return Err(crate::LabError::Derivation(
                "unknown address format".to_string(),
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
    let bip44_path = format!("m/44'/1'/{account}'/0/{index}");
    let bip49_path = format!("m/49'/1'/{account}'/0/{index}");
    let bip84_path = format!("m/84'/1'/{account}'/0/{index}");

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
