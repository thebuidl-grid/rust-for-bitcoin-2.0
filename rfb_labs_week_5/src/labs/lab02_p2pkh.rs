//! Lab 02 — construct and explain legacy P2PKH.

use std::str::FromStr;

use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::error::LabError;
use crate::model::P2pkhSpendTemplate;
use crate::LabResult;

/// Parse a serialized (hex) public key.
fn parse_public_key(public_key_hex: &str) -> LabResult<PublicKey> {
    PublicKey::from_str(public_key_hex.trim())
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;
    Ok(Address::p2pkh(public_key, network).to_string())
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;
    Ok(ScriptBuf::new_p2pkh(&public_key.pubkey_hash()).to_hex_string())
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;
    Ok(public_key.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    hex::decode(signature_hex.trim())
        .map_err(|error| LabError::InvalidScript(format!("signature: {error}")))?;
    let public_key = parse_public_key(public_key_hex)?;

    Ok(P2pkhSpendTemplate {
        // Legacy inputs carry <sig> <pubkey> in the ScriptSig; the witness is empty.
        script_sig_items: vec![signature_hex.trim().to_owned(), public_key.to_string()],
        witness_items: Vec::new(),
    })
}
