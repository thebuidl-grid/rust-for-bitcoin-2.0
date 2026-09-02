//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::key::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};
use std::str::FromStr;

use crate::model::{AddressFormat, DerivedAddressSet};
use crate::{LabError, LabResult};

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    // todo!("Lab 10: derive one P2PKH, wrapped P2WPKH, or native P2WPKH address")

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

    let secp_pk = child.private_key.public_key(&secp);
    let compressed = CompressedPublicKey(secp_pk);

    match format {
        AddressFormat::P2pkh => Ok(Address::p2pkh(&PublicKey::new(secp_pk), network).to_string()),
        AddressFormat::P2wpkh => Ok(Address::p2wpkh(&compressed, network).to_string()),
        AddressFormat::P2sh => Ok(Address::p2shwpkh(&compressed, network).to_string()),
        _ => Err(LabError::InvalidAddress("unsupported format".into())),
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
    // todo!("Lab 10: derive three address families from one recovery root")
    let coin = if network == Network::Bitcoin { 0 } else { 1 };

    let path_1 = format!("m/44'/{coin}'/{account}'/0/{index}");
    let path_2 = format!("m/49'/{coin}'/{account}'/0/{index}");
    let path_3 = format!("m/84'/{coin}'/{account}'/0/{index}");

    let bip44_p2pkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &*path_1,
        AddressFormat::P2pkh,
        network,
    )?;
    let bip49_p2sh_p2wpkh =
        derive_address_for_path(mnemonic, passphrase, &*path_2, AddressFormat::P2sh, network)?;
    let bip84_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &*path_3,
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
    // todo!("Lab 10: derive twice and compare the results")
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
    // todo!("Lab 10: compare addresses selected by two child indexes")
    let first = derive_address_for_path(mnemonic, passphrase, first_path, format, network)?;
    let second = derive_address_for_path(mnemonic, passphrase, second_path, format, network)?;
    Ok(first != second)
}
