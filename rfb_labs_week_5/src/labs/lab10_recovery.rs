//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};

use crate::error::LabError;
use crate::model::{AddressFormat, DerivedAddressSet};
use crate::LabResult;

fn derive_child_xpriv(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<Xpriv> {
    let parsed_mnemonic = mnemonic
        .parse::<Mnemonic>()
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = parsed_mnemonic.to_seed(passphrase);

    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let secp = Secp256k1::new();
    master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(e.to_string()))
}

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let child_xpriv = derive_child_xpriv(mnemonic, passphrase, path, network)?;
    let secp = Secp256k1::new();
    let public = PublicKey::new(child_xpriv.private_key.public_key(&secp));

    match format {
        AddressFormat::P2pkh => Ok(Address::p2pkh(public, network).to_string()),
        AddressFormat::P2wpkh => {
            let compressed = CompressedPublicKey::try_from(public)
                .map_err(|e| LabError::InvalidKey(e.to_string()))?;
            Ok(Address::p2wpkh(&compressed, network).to_string())
        }
        AddressFormat::P2sh => {
            let compressed = CompressedPublicKey::try_from(public)
                .map_err(|e| LabError::InvalidKey(e.to_string()))?;
            Ok(Address::p2shwpkh(&compressed, network).to_string())
        }
        AddressFormat::P2tr | AddressFormat::Unknown => Err(LabError::InvalidPath(
            "Lab 10 supports P2PKH, P2SH-wrapped P2WPKH, and native P2WPKH only".to_owned(),
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
    let bip44_path = format!("m/44'/1'/{account}'/0/{index}");
    let bip49_path = format!("m/49'/1'/{account}'/0/{index}");
    let bip84_path = format!("m/84'/1'/{account}'/0/{index}");

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
