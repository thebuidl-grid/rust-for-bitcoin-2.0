//! Lab 04 — construct and explain native SegWit P2WPKH.

use std::str::FromStr;

use bitcoin::{Address, CompressedPublicKey, Network, PublicKey, ScriptBuf, WitnessProgram};

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::{LabError, LabResult};

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let compressed = parse_compressed_public_key(public_key_hex)?;

    Ok(Address::p2wpkh(&compressed, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let compressed = parse_compressed_public_key(public_key_hex)?;

    // Serializes as 0014<hash>: OP_0 followed by a 20-byte push. There is no
    // OP_DUP/OP_CHECKSIG here, because the spending rules live in the witness.
    Ok(ScriptBuf::new_p2wpkh(&compressed.wpubkey_hash()).to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let compressed = parse_compressed_public_key(public_key_hex)?;
    let program = WitnessProgram::p2wpkh(&compressed);

    Ok(WitnessProgramReport {
        version: program.version().to_num(),
        program_hex: hex::encode(program.program().as_bytes()),
        program_length: program.program().len(),
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    let public_key = parse_public_key(public_key_hex)?;
    // Reject an uncompressed key before modelling the spend: BIP143 only defines
    // P2WPKH for compressed keys.
    CompressedPublicKey::try_from(public_key)
        .map_err(|error| LabError::InvalidKey(error.to_string()))?;
    let signature = normalize_hex(signature_hex)?;

    Ok(NativeSegwitSpend {
        // Native SegWit inputs must have a completely empty ScriptSig; a non-empty
        // one makes the input non-standard and breaks the txid malleability fix.
        script_sig_hex: String::new(),
        witness_items: vec![signature, public_key.to_string()],
    })
}

/// Parse a public key from hex.
fn parse_public_key(public_key_hex: &str) -> LabResult<PublicKey> {
    PublicKey::from_str(public_key_hex.trim())
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Parse a public key from hex and require the compressed 33-byte encoding.
fn parse_compressed_public_key(public_key_hex: &str) -> LabResult<CompressedPublicKey> {
    CompressedPublicKey::try_from(parse_public_key(public_key_hex)?)
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Validate a hex push and return it in canonical lower case.
fn normalize_hex(value: &str) -> LabResult<String> {
    let bytes =
        hex::decode(value.trim()).map_err(|error| LabError::InvalidScript(error.to_string()))?;

    Ok(hex::encode(bytes))
}
