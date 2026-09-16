//! Lab 04 — construct and explain native SegWit P2WPKH.

use bitcoin::hashes::Hash;
use bitcoin::script::Builder;
use bitcoin::CompressedPublicKey;
use bitcoin::Network;
use bitcoin::PublicKey;
use std::str::FromStr;

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::LabResult;

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let pubkey = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::LabError::InvalidKey(e.to_string()))?;

    let compressed = CompressedPublicKey::try_from(pubkey)
        .map_err(|_| crate::LabError::InvalidKey("Key must be compressed".to_string()))?;
    let address = bitcoin::Address::p2wpkh(&compressed, network);

    Ok(address.to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let pubkey = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::LabError::InvalidKey(e.to_string()))?;

    let hash = pubkey.pubkey_hash();
    let script = Builder::new()
        .push_int(0)
        .push_slice(hash.as_byte_array())
        .into_script();

    Ok(script.to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let pubkey = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::LabError::InvalidKey(e.to_string()))?;

    Ok(WitnessProgramReport {
        version: 0u8,
        program_hex: pubkey.pubkey_hash().to_string(),
        program_length: 20,
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    Ok(NativeSegwitSpend {
        script_sig_hex: String::new(),
        witness_items: vec![signature_hex.to_string(), public_key_hex.to_string()],
    })
}
