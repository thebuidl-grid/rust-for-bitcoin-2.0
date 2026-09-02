//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::key::Secp256k1;
use bitcoin::{Address, Network};

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
    let parsed =
        Mnemonic::from_str(mnemonic).map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = parsed.to_seed(passphrase);
    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let secp = Secp256k1::new();
    let child_xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    let child_xpub = Xpub::from_priv(&secp, &child_xpriv);
    let compressed = child_xpub.to_pub();

    let address = match format {
        AddressFormat::P2sh => {
            let wpkh_script = Address::p2wpkh(&compressed, network).script_pubkey();
            Address::p2sh(&wpkh_script, network)
                .map_err(|e| LabError::InvalidScript(e.to_string()))?
                .to_string()
        }
        AddressFormat::P2pkh => Address::p2pkh(compressed, network).to_string(),
        AddressFormat::P2wpkh => Address::p2wpkh(&compressed, network).to_string(),
        _ => return Err(LabError::InvalidAddress("Invalid address".to_string())),
    };

    Ok(address)
}

/// Derive index `n` on the BIP44, BIP49, and BIP84 receive branches.
pub fn derive_address_set(
    mnemonic: &str,
    passphrase: &str,
    account: u32,
    index: u32,
    network: Network,
) -> LabResult<DerivedAddressSet> {
    // BIP44: m/44'/0'/account'/0/index (Legacy P2PKH)
    let bip44_path = format!("m/44'/0'/{account}'/0/{index}");
    let bip44_p2pkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip44_path,
        AddressFormat::P2pkh,
        network,
    )?;

    // BIP49: m/49'/0'/account'/0/index (Wrapped SegWit P2SH-P2WPKH)
    let bip49_path = format!("m/49'/0'/{account}'/0/{index}");
    let bip49_p2sh_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip49_path,
        AddressFormat::P2sh,
        network,
    )?;

    // BIP84: m/84'/0'/account'/0/index (Native SegWit P2WPKH)
    let bip84_path = format!("m/84'/0'/{account}'/0/{index}");
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
    let first_derivation = derive_address_for_path(mnemonic, passphrase, path, format, network)?;
    let second_derivation = derive_address_for_path(mnemonic, passphrase, path, format, network)?;

    Ok(first_derivation == second_derivation)
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
    let first_address = derive_address_for_path(mnemonic, passphrase, first_path, format, network)?;
    let second_address =
        derive_address_for_path(mnemonic, passphrase, second_path, format, network)?;

    Ok(first_address != second_address)
}
