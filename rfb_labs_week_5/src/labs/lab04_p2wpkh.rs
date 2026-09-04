//! Lab 04 — construct and explain native SegWit P2WPKH.

use std::str::FromStr;

use bitcoin::{Address, CompressedPublicKey, Network, PublicKey, ScriptBuf};

use crate::error::LabError;
use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::LabResult;

/// Parse a serialized public key and require the compressed encoding SegWit needs.
fn parse_compressed(public_key_hex: &str) -> LabResult<CompressedPublicKey> {
    let public_key = PublicKey::from_str(public_key_hex.trim())
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;
    CompressedPublicKey::try_from(public_key)
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let compressed = parse_compressed(public_key_hex)?;
    Ok(Address::p2wpkh(&compressed, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let compressed = parse_compressed(public_key_hex)?;
    Ok(ScriptBuf::new_p2wpkh(&compressed.wpubkey_hash()).to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let compressed = parse_compressed(public_key_hex)?;
    let program = compressed.wpubkey_hash();
    let program_hex = program.to_string();
    Ok(WitnessProgramReport {
        version: 0,
        program_length: program_hex.len() / 2,
        program_hex,
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    hex::decode(signature_hex.trim())
        .map_err(|error| LabError::InvalidScript(format!("signature: {error}")))?;
    let public_key = PublicKey::from_str(public_key_hex.trim())
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;

    Ok(NativeSegwitSpend {
        // Native SegWit moves the unlocking data into the witness; ScriptSig stays empty.
        script_sig_hex: String::new(),
        witness_items: vec![signature_hex.trim().to_owned(), public_key.to_string()],
    })
}
