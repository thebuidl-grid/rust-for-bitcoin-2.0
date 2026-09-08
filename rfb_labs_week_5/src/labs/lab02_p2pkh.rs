//! Lab 02 — construct and explain legacy P2PKH.

use bitcoin::{Network, PublicKey, Address};
use hex::ToHex;
use std::str::FromStr;

use crate::model::P2pkhSpendTemplate;
use crate::LabResult;
use crate::error::LabError;

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {

    let public_key = PublicKey::from_str(public_key_hex)
    .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let p2pkh_address = Address::p2pkh(public_key, network).to_string();

    Ok(p2pkh_address)
    // todo!("Lab 02: hash the public key and encode a P2PKH address")
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {

    let public_key = PublicKey::from_str(public_key_hex)
    .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let p2pkh_address = Address::p2pkh(public_key, Network::Bitcoin);

    let script_pub_key = p2pkh_address.script_pubkey().to_hex_string();

    Ok(script_pub_key)

    // todo!("Lab 02: build OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG")
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    let public_key = PublicKey::from_str(public_key_hex).map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let public_keyhash = public_key.pubkey_hash().to_string();

    Ok(public_keyhash)

    // todo!("Lab 02: calculate the public-key hash committed by P2PKH")
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template( signature_hex: &str, public_key_hex: &str,) -> LabResult<P2pkhSpendTemplate> {
    let signature = signature_hex.to_string();
    let public_key = public_key_hex.to_string();

    let unlocking_template = P2pkhSpendTemplate {
        script_sig_items: vec![signature, public_key],
        witness_items: vec![]
    };

    Ok(unlocking_template)

    // todo!("Lab 02: model P2PKH ScriptSig items and its empty witness")
}
