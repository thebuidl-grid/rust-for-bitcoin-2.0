//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::key::PublicKey;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network};
use std::str::FromStr;

use crate::error::LabError;
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
    let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, mnemonic)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let master = Xpriv::new_master(network, &mnemonic.to_seed(passphrase))
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    let path = DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let secp = Secp256k1::new();
    let child = master
        .derive_priv(&secp, &path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    let public = PublicKey::new(child.private_key.public_key(&secp));
    let compressed =
        CompressedPublicKey::try_from(public).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    let native = Address::p2wpkh(&compressed, network);
    Ok(match format {
        AddressFormat::P2pkh => Address::p2pkh(&compressed, network).to_string(),
        AddressFormat::P2sh => Address::p2sh(&native.script_pubkey(), network)
            .map_err(|e| LabError::InvalidScript(e.to_string()))?
            .to_string(),
        AddressFormat::P2wpkh => native.to_string(),
        _ => {
            return Err(LabError::InvalidAddress(
                "unsupported address format".to_owned(),
            ))
        }
    })
}

/// Derive index `n` on the BIP44, BIP49, and BIP84 receive branches.
pub fn derive_address_set(
    mnemonic: &str,
    passphrase: &str,
    account: u32,
    index: u32,
    network: Network,
) -> LabResult<DerivedAddressSet> {
    Ok(DerivedAddressSet {
        bip44_p2pkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &format!("m/44'/1'/{}'/0/{}", account, index),
            AddressFormat::P2pkh,
            network,
        )?,
        bip49_p2sh_p2wpkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &format!("m/49'/1'/{}'/0/{}", account, index),
            AddressFormat::P2sh,
            network,
        )?,
        bip84_p2wpkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &format!("m/84'/1'/{}'/0/{}", account, index),
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
    Ok(
        derive_address_for_path(mnemonic, passphrase, path, format, network)?
            == derive_address_for_path(mnemonic, passphrase, path, format, network)?,
    )
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
    Ok(
        derive_address_for_path(mnemonic, passphrase, first_path, format, network)?
            != derive_address_for_path(mnemonic, passphrase, second_path, format, network)?,
    )
}
