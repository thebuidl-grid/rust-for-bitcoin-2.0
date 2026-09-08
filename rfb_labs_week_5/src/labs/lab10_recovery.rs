//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::{CompressedPublicKey, Network, PublicKey};
use std::str::FromStr;

use crate::model::{AddressFormat, DerivedAddressSet};
use crate::LabResult;
use crate::LabError;

/// Derive a child key at `path` and return the secp256k1 public key.
fn derive_public_key(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<PublicKey> {
    let m = Mnemonic::parse_normalized(mnemonic)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = m.to_seed(passphrase);

    let master = Xpriv::new_master(network, &seed)
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let child = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;

    Ok(PublicKey::new(child.private_key.public_key(&secp)))
}

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let public_key = derive_public_key(mnemonic, passphrase, path, network)?;

    let address = match format {
        AddressFormat::P2pkh => bitcoin::Address::p2pkh(public_key, network).to_string(),
        AddressFormat::P2sh => {
            let compressed = CompressedPublicKey::try_from(public_key)
                .map_err(|e| LabError::InvalidKey(e.to_string()))?;
            let secp = bitcoin::secp256k1::Secp256k1::new();
            bitcoin::Address::p2shwpkh(&compressed, network).to_string()
        }
        AddressFormat::P2wpkh => {
            let compressed = CompressedPublicKey::try_from(public_key)
                .map_err(|e| LabError::InvalidKey(e.to_string()))?;
            bitcoin::Address::p2wpkh(&compressed, network).to_string()
        }
        AddressFormat::P2tr => {
            let secp = bitcoin::secp256k1::Secp256k1::new();
            let xonly = bitcoin::XOnlyPublicKey::from(public_key.inner);
            bitcoin::Address::p2tr(&secp, xonly, None, network).to_string()
        }
        AddressFormat::Unknown => {
            return Err(LabError::InvalidAddress("unknown address format".to_string()))
        }
    };

    Ok(address)
}

/// Derive index `n` on the BIP44, BIP49, and BIP84 receive branches.
/// Uses coin_type 1 (testnet) for regtest compatibility.
pub fn derive_address_set(
    mnemonic: &str,
    passphrase: &str,
    account: u32,
    index: u32,
    network: Network,
) -> LabResult<DerivedAddressSet> {
    let bip44_path = format!("m/44'/1'/{}'/ 0/{}", account, index).replace(" ", "");
    let bip49_path = format!("m/49'/1'/{}'/ 0/{}", account, index).replace(" ", "");
    let bip84_path = format!("m/84'/1'/{}'/ 0/{}", account, index).replace(" ", "");

    let bip44_p2pkh = derive_address_for_path(mnemonic, passphrase, &bip44_path, AddressFormat::P2pkh, network)?;
    let bip49_p2sh_p2wpkh = derive_address_for_path(mnemonic, passphrase, &bip49_path, AddressFormat::P2sh, network)?;
    let bip84_p2wpkh = derive_address_for_path(mnemonic, passphrase, &bip84_path, AddressFormat::P2wpkh, network)?;

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
