//! Lab 02 — construct and explain legacy P2PKH.

use std::str::FromStr;

use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::model::P2pkhSpendTemplate;
use crate::{LabError, LabResult};

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;

    Ok(Address::p2pkh(public_key, network).to_string())
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;

    // OP_DUP OP_HASH160 <20-byte pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG.
    Ok(ScriptBuf::new_p2pkh(&public_key.pubkey_hash()).to_hex_string())
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;

    // RIPEMD160(SHA256(pubkey)), which is what the output actually commits to. The
    // public key itself stays hidden until the input that spends it is published.
    Ok(public_key.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    let public_key = parse_public_key(public_key_hex)?;
    let signature = normalize_hex(signature_hex)?;

    Ok(P2pkhSpendTemplate {
        // The signature is pushed first so it sits under the public key on the stack.
        script_sig_items: vec![signature, public_key.to_string()],
        // Legacy inputs carry no witness field at all.
        witness_items: Vec::new(),
    })
}

/// Parse a serialized public key in either compressed or uncompressed form.
fn parse_public_key(public_key_hex: &str) -> LabResult<PublicKey> {
    PublicKey::from_str(public_key_hex.trim())
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Validate a hex push and return it in canonical lower case.
fn normalize_hex(value: &str) -> LabResult<String> {
    let bytes =
        hex::decode(value.trim()).map_err(|error| LabError::InvalidScript(error.to_string()))?;

    Ok(hex::encode(bytes))
}
