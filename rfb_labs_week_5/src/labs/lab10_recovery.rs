//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bitcoin::bip32::{ChildNumber, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};

use crate::labs::lab08_bip32::{derive_xpriv_at_path, parse_derivation_path};
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
    let (xpriv, _) = derive_xpriv_at_path(mnemonic, passphrase, path, network)?;
    let secp = Secp256k1::new();
    let public_key = Xpub::from_priv(&secp, &xpriv).to_pub();

    let address = match format {
        AddressFormat::P2pkh => Address::p2pkh(public_key, network),
        AddressFormat::P2sh => Address::p2shwpkh(&public_key, network),
        AddressFormat::P2wpkh => Address::p2wpkh(&public_key, network),
        AddressFormat::P2tr => {
            return Err(LabError::InvalidAddress(
                "Taproot derivation is outside this recovery lab's scope".to_owned(),
            ));
        }
        AddressFormat::Unknown => {
            return Err(LabError::InvalidAddress(
                "cannot derive an address for an unknown format".to_owned(),
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
    ChildNumber::from_hardened_idx(account)
        .map_err(|err| LabError::InvalidPath(err.to_string()))?;
    ChildNumber::from_normal_idx(index).map_err(|err| LabError::InvalidPath(err.to_string()))?;

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
    let first_derivation_path = parse_derivation_path(first_path)?;
    let second_derivation_path = parse_derivation_path(second_path)?;
    let first_levels = first_derivation_path.as_ref();
    let second_levels = second_derivation_path.as_ref();

    if first_levels.is_empty()
        || first_levels.len() != second_levels.len()
        || first_levels[..first_levels.len() - 1] != second_levels[..second_levels.len() - 1]
    {
        return Err(LabError::InvalidPath(
            "the paths must differ only at their final address index".to_owned(),
        ));
    }

    match (first_levels.last(), second_levels.last()) {
        (
            Some(ChildNumber::Normal { index: first_index }),
            Some(ChildNumber::Normal {
                index: second_index,
            }),
        ) if first_index != second_index => {}
        _ => return Ok(false),
    }

    let first = derive_address_for_path(mnemonic, passphrase, first_path, format, network)?;
    let second = derive_address_for_path(mnemonic, passphrase, second_path, format, network)?;

    Ok(first != second)
}
