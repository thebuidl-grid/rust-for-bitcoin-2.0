//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::key::UntweakedPublicKey;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network};

use crate::model::{AddressFormat, DerivedAddressSet};
use crate::{LabError, LabResult};

/// Derive one address from an arbitrary full path and selected script family.
///
/// Recovery does not depend only on the mnemonic, passphrase, and path — the *script
/// family* (P2PKH vs. wrapped P2WPKH vs. native P2WPKH vs. Taproot) is a wallet
/// convention layered on top of the same key material, and it must also match or the
/// restored addresses will not be the ones funds were sent to.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let parsed =
        Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    let seed = parsed.to_seed(passphrase);
    let derivation_path = path
        .parse::<DerivationPath>()
        .map_err(|error| LabError::InvalidPath(error.to_string()))?;

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network, &seed)
        .map_err(|error| LabError::Derivation(error.to_string()))?;
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|error| LabError::Derivation(error.to_string()))?;

    let private_key = child.to_priv();
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
            let internal_key = UntweakedPublicKey::from(public_key.inner);
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
    let coin_type = if network == Network::Bitcoin { 0 } else { 1 };

    let bip44_p2pkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &format!("m/44'/{coin_type}'/{account}'/0/{index}"),
        AddressFormat::P2pkh,
        network,
    )?;
    let bip49_p2sh_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &format!("m/49'/{coin_type}'/{account}'/0/{index}"),
        AddressFormat::P2sh,
        network,
    )?;
    let bip84_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &format!("m/84'/{coin_type}'/{account}'/0/{index}"),
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
