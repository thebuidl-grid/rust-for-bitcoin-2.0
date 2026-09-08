//! Lab 02 — construct and explain legacy P2PKH.

use bitcoin::{Address, Network, PublicKey, ScriptBuf};
use std::str::FromStr;

use crate::error::LabError;
use crate::model::P2pkhSpendTemplate;
use crate::LabResult;

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let pk =
        PublicKey::from_str(public_key_hex).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    let address = Address::p2pkh(pk, network);
    Ok(address.to_string())
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let pk =
        PublicKey::from_str(public_key_hex).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    let script = ScriptBuf::new_p2pkh(&pk.pubkey_hash());
    Ok(script.to_hex_string())
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    let pk =
        PublicKey::from_str(public_key_hex).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    Ok(pk.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    let pk =
        PublicKey::from_str(public_key_hex).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    hex::decode(signature_hex).map_err(|e| LabError::InvalidScript(e.to_string()))?;

    Ok(P2pkhSpendTemplate {
        script_sig_items: vec![signature_hex.to_owned(), pk.to_string()],
        witness_items: vec![],
    })
}
