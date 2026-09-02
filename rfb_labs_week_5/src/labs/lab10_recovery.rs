//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network};

use crate::labs::lab08_bip32::{derive_xpriv, parse_path};
use crate::model::{AddressFormat, DerivedAddressSet};
use crate::{LabError, LabResult};

/// SLIP44 registers coin type 0 for Bitcoin mainnet and 1 for every test chain.
const MAINNET_COIN_TYPE: u32 = 0;
const TESTNET_COIN_TYPE: u32 = 1;

/// The external branch that receives payments, as opposed to `1` for change.
const RECEIVE_BRANCH: u32 = 0;

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let secp = Secp256k1::new();
    let derivation_path = parse_path(path)?;
    let child = derive_xpriv(mnemonic, passphrase, &derivation_path, network)?;
    let public_key = child.to_priv().public_key(&secp);
    let compressed = CompressedPublicKey::try_from(public_key)
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;

    // The path selects the key and the format selects the script, so the same key can
    // be locked four different ways. A mnemonic alone does not fix the address.
    let address = match format {
        AddressFormat::P2pkh => Address::p2pkh(public_key, network),
        AddressFormat::P2sh => Address::p2shwpkh(&compressed, network),
        AddressFormat::P2wpkh => Address::p2wpkh(&compressed, network),
        AddressFormat::P2tr => {
            Address::p2tr(&secp, public_key.inner.x_only_public_key().0, None, network)
        }
        AddressFormat::Unknown => {
            return Err(LabError::InvalidAddress(
                "cannot derive an address for an unknown script family".to_owned(),
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
    let branch = |purpose: u32| receive_path(purpose, account, index, network);

    Ok(DerivedAddressSet {
        bip44_p2pkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &branch(44),
            AddressFormat::P2pkh,
            network,
        )?,
        bip49_p2sh_p2wpkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &branch(49),
            AddressFormat::P2sh,
            network,
        )?,
        bip84_p2wpkh: derive_address_for_path(
            mnemonic,
            passphrase,
            &branch(84),
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

/// Build `m/purpose'/coin'/account'/0/index` for the selected network.
fn receive_path(purpose: u32, account: u32, index: u32, network: Network) -> String {
    let coin_type = match network {
        Network::Bitcoin => MAINNET_COIN_TYPE,
        _ => TESTNET_COIN_TYPE,
    };

    format!("m/{purpose}'/{coin_type}'/{account}'/{RECEIVE_BRANCH}/{index}")
}
