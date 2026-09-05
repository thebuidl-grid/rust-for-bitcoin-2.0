//! Lab 04 — construct and explain native SegWit P2WPKH.

use std::str::FromStr;

use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::{LabError, LabResult};

fn parse_compressed_key(public_key_hex: &str) -> LabResult<CompressedPublicKey> {
    let public_key = PublicKey::from_str(public_key_hex)
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;
    CompressedPublicKey::try_from(public_key)
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let compressed = parse_compressed_key(public_key_hex)?;
    Ok(Address::p2wpkh(&compressed, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let compressed = parse_compressed_key(public_key_hex)?;
    // The scriptPubKey only commits to the witness program, so any network's HRP works here.
    Ok(Address::p2wpkh(&compressed, Network::Bitcoin)
        .script_pubkey()
        .to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let compressed = parse_compressed_key(public_key_hex)?;
    let address = Address::p2wpkh(&compressed, Network::Bitcoin);
    let program = address
        .witness_program()
        .ok_or_else(|| LabError::InvalidScript("address is not a witness program".to_owned()))?;
    let bytes = program.program().as_bytes();

    Ok(WitnessProgramReport {
        version: program.version().to_num(),
        program_hex: hex::encode(bytes),
        program_length: bytes.len(),
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    parse_compressed_key(public_key_hex)?;

    Ok(NativeSegwitSpend {
        script_sig_hex: String::new(),
        witness_items: vec![signature_hex.to_owned(), public_key_hex.to_owned()],
    })
}
