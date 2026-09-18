//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::CompressedPublicKey;
use bitcoin::Network;
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
    let mnemonic =
        Mnemonic::parse(mnemonic).map_err(|e| crate::LabError::InvalidMnemonic(e.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);
    let master_xpriv = Xpriv::new_master(network, &seed)
        .map_err(|e| crate::LabError::Derivation(e.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| crate::LabError::InvalidPath(e.to_string()))?;

    let derived_xpriv = master_xpriv
        .derive_priv(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path)
        .map_err(|e| crate::LabError::Derivation(e.to_string()))?;

    let pubkey = derived_xpriv
        .to_priv()
        .public_key(&bitcoin::secp256k1::Secp256k1::new());

    let address = match format {
        AddressFormat::P2pkh => bitcoin::Address::p2pkh(&pubkey, network).to_string(),
        AddressFormat::P2wpkh => {
            let compressed = CompressedPublicKey::try_from(pubkey)
                .map_err(|_| crate::LabError::InvalidKey("Key must be compressed".to_string()))?;
            bitcoin::Address::p2wpkh(&compressed, network).to_string()
        }
        AddressFormat::P2sh => {
            // P2SH-P2WPKH (wrapped SegWit)
            let compressed = CompressedPublicKey::try_from(pubkey)
                .map_err(|_| crate::LabError::InvalidKey("Key must be compressed".to_string()))?;
            let witness_script = bitcoin::ScriptBuf::new_p2wpkh(&compressed.wpubkey_hash());
            bitcoin::Address::p2sh(&witness_script, network)
                .map_err(|e| crate::LabError::InvalidScript(e.to_string()))?
                .to_string()
        }
        _ => {
            return Err(crate::LabError::InvalidScript(
                "Unsupported address format".to_string(),
            ))
        }
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
    let bip44_path = format!("m/44'/0'/{account}'/0/{index}");
    let bip49_path = format!("m/49'/0'/{account}'/0/{index}");
    let bip84_path = format!("m/84'/0'/{account}'/0/{index}");

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
    let first_address = derive_address_for_path(mnemonic, passphrase, path, format, network)?;
    let second_address = derive_address_for_path(mnemonic, passphrase, path, format, network)?;

    Ok(first_address == second_address)
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
