//! Lab 02 — construct and explain legacy P2PKH.

use crate::model::P2pkhSpendTemplate;
use crate::{LabError, LabResult};
use bitcoin::{Address, Network, PublicKey};
use std::str::FromStr;

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    // todo!("Lab 02: hash the public key and encode a P2PKH address")
    let parsed_key = PublicKey::from_str(public_key_hex);
    match parsed_key {
        Ok(key) => {
            let address = Address::p2pkh(&key, network).to_string();
            Ok(address)
        }
        Err(msg) => Err(LabError::InvalidKey(public_key_hex.to_string())),
    }
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    // todo!("Lab 02: build OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG")
    let public_key = PublicKey::from_str(public_key_hex)
        .map_err(|_| LabError::InvalidKey(public_key_hex.to_string()))?;

    let hash = public_key.pubkey_hash();

    let res = Address::p2pkh(&public_key, Network::Regtest)
        .script_pubkey()
        .to_hex_string();
    Ok(res)
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    // todo!("Lab 02: calculate the public-key hash committed by P2PKH")
    let public_key = PublicKey::from_str(public_key_hex)
        .map_err(|_| LabError::InvalidKey(public_key_hex.to_string()))?;

    Ok(public_key.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    // todo!("Lab 02: model P2PKH ScriptSig items and its empty witness")
    Ok(P2pkhSpendTemplate {
        script_sig_items: vec![signature_hex.to_string(), public_key_hex.to_string()],
        witness_items: Vec::new(),
    })
}
