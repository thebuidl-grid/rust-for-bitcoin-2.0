//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network};

use crate::error::LabError;
use crate::labs::common::derive_xpriv;
use crate::model::{AddressFormat, DerivedAddressSet};
use crate::LabResult;

/// Coin type used by the class labs: `0` on mainnet, `1` everywhere else.
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
    let secp = Secp256k1::new();
    let xpriv = derive_xpriv(mnemonic, passphrase, path, network)?;
    let private_key = xpriv.to_priv();
    let public_key = private_key.public_key(&secp);

    let address = match format {
        AddressFormat::P2pkh => Address::p2pkh(public_key, network),
        AddressFormat::P2sh => {
            let compressed = CompressedPublicKey::try_from(public_key)
                .map_err(|error| LabError::InvalidKey(error.to_string()))?;
            Address::p2shwpkh(&compressed, network)
        }
        AddressFormat::P2wpkh => {
            let compressed = CompressedPublicKey::try_from(public_key)
                .map_err(|error| LabError::InvalidKey(error.to_string()))?;
            Address::p2wpkh(&compressed, network)
        }
        AddressFormat::P2tr => {
            let (internal_key, _parity) = xpriv.private_key.public_key(&secp).x_only_public_key();
            Address::p2tr(&secp, internal_key, None, network)
        }
        AddressFormat::Unknown => {
            return Err(LabError::InvalidScript(
                "cannot derive an address for an unknown format".to_owned(),
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
    let receive = |purpose: u32| format!("m/{purpose}'/{coin}'/{account}'/0/{index}");

    Ok(DerivedAddressSet {
        bip44_p2pkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &receive(44),
            AddressFormat::P2pkh,
            network,
        )?,
        bip49_p2sh_p2wpkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &receive(49),
            AddressFormat::P2sh,
            network,
        )?,
        bip84_p2wpkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &receive(84),
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
