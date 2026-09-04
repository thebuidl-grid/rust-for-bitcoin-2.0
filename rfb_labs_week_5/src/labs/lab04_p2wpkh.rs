//! Lab 04 — construct and explain native SegWit P2WPKH.

use std::str::FromStr;

use bitcoin::hashes::Hash;
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey, ScriptBuf};

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::{LabError, LabResult};

fn parse_compressed_public_key(public_key_hex: &str) -> LabResult<CompressedPublicKey> {
    let public = PublicKey::from_str(public_key_hex)
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;
    CompressedPublicKey::try_from(public).map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let compressed = parse_compressed_public_key(public_key_hex)?;
    Ok(Address::p2wpkh(&compressed, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let compressed = parse_compressed_public_key(public_key_hex)?;
    Ok(ScriptBuf::new_p2wpkh(&compressed.wpubkey_hash()).to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let compressed = parse_compressed_public_key(public_key_hex)?;
    let hash = compressed.wpubkey_hash();
    Ok(WitnessProgramReport {
        version: 0,
        program_hex: hash.to_string(),
        program_length: hash.as_byte_array().len(),
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    parse_compressed_public_key(public_key_hex)?;
    Ok(NativeSegwitSpend {
        script_sig_hex: String::new(),
        witness_items: vec![signature_hex.to_owned(), public_key_hex.to_owned()],
    })
}
