//! Lab 04 — construct and explain native SegWit P2WPKH.

use std::str::FromStr;

use bitcoin::{Address, CompressedPublicKey, Network, PublicKey, ScriptBuf};

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::{LabError, LabResult};

fn parse_compressed(public_key_hex: &str) -> LabResult<CompressedPublicKey> {
    let public_key =
        PublicKey::from_str(public_key_hex).map_err(|error| LabError::InvalidKey(format!("{error}")))?;
    CompressedPublicKey::try_from(public_key).map_err(|error| LabError::InvalidKey(format!("{error}")))
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
    let script = ScriptBuf::new_p2wpkh(&compressed.wpubkey_hash());
    let version = script
        .witness_version()
        .ok_or_else(|| LabError::InvalidScript("scriptPubKey has no witness version".to_owned()))?
        .to_num();
    let program = &script.as_bytes()[2..];
    Ok(WitnessProgramReport {
        version,
        program_hex: hex::encode(program),
        program_length: program.len(),
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    parse_compressed(public_key_hex)?;
    Ok(NativeSegwitSpend {
        script_sig_hex: String::new(),
        witness_items: vec![signature_hex.to_owned(), public_key_hex.to_owned()],
    })
}
