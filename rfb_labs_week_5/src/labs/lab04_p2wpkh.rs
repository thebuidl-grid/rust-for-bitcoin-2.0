//! Lab 04 — construct and explain native SegWit P2WPKH.

use bitcoin::Network;

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::LabResult;

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    use std::str::FromStr;
    let pubkey = bitcoin::PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    let compressed = bitcoin::CompressedPublicKey::try_from(pubkey)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    Ok(bitcoin::Address::p2wpkh(&compressed, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    use std::str::FromStr;
    let pubkey = bitcoin::PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    let compressed = bitcoin::CompressedPublicKey::try_from(pubkey)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    Ok(bitcoin::Address::p2wpkh(&compressed, bitcoin::Network::Bitcoin)
        .script_pubkey()
        .to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    use std::str::FromStr;
    let pubkey = bitcoin::PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    let compressed = bitcoin::CompressedPublicKey::try_from(pubkey)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    let program = bitcoin::blockdata::script::witness_program::WitnessProgram::p2wpkh(&compressed);
    Ok(WitnessProgramReport {
        version: program.version().to_num(),
        program_hex: hex::encode(program.program().as_bytes()),
        program_length: program.program().as_bytes().len(),
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    use std::str::FromStr;
    let _ = bitcoin::PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    Ok(NativeSegwitSpend {
        script_sig_hex: "".to_owned(),
        witness_items: vec![signature_hex.to_owned(), public_key_hex.to_owned()],
    })
}

