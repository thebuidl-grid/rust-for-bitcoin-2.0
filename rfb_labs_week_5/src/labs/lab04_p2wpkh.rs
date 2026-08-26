//! Lab 04 — construct and explain native SegWit P2WPKH.

use bitcoin::{Address, CompressedPublicKey, Network, ScriptBuf};

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::{LabError, LabResult};

fn parse_compressed_public_key(public_key_hex: &str) -> LabResult<CompressedPublicKey> {
    public_key_hex
        .parse::<CompressedPublicKey>()
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key = parse_compressed_public_key(public_key_hex)?;
    Ok(Address::p2wpkh(&public_key, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let public_key = parse_compressed_public_key(public_key_hex)?;
    let script = ScriptBuf::new_p2wpkh(&public_key.wpubkey_hash());
    Ok(script.to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let public_key = parse_compressed_public_key(public_key_hex)?;
    let program_hex = public_key.wpubkey_hash().to_string();

    Ok(WitnessProgramReport {
        version: 0,
        program_length: program_hex.len() / 2,
        program_hex,
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
///
/// Unlike P2PKH (unlocking data in ScriptSig) or P2SH-wrapped SegWit (a thin ScriptSig
/// that only pushes the witness program, with the real unlocking data still in the
/// witness), native P2WPKH leaves ScriptSig entirely empty.
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
