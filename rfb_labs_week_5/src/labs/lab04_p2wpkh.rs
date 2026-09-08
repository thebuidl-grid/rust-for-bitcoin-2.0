//! Lab 04 — construct and explain native SegWit P2WPKH.

use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};
use std::str::FromStr;

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::LabResult;
use crate::LabError;

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {

    let public_key = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;

    let compressed_public_key = CompressedPublicKey::try_from(public_key)
                                                                .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let p2wpkh_address = Address::p2wpkh(&compressed_public_key, network);

    Ok(p2wpkh_address.to_string())

    // todo!("Lab 04: encode a version-0, 20-byte witness program")
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    
     let public_key = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;

    let compressed_public_key = CompressedPublicKey::try_from(public_key)
                                                                .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let p2wpkh_address = Address::p2wpkh(&compressed_public_key, Network::Bitcoin);

    let x = p2wpkh_address.script_pubkey().witness_version();

    Ok(p2wpkh_address.script_pubkey().to_hex_string())

    // todo!("Lab 04: construct the native SegWit scriptPubKey")
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    
      let public_key = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;

    let compressed_public_key = CompressedPublicKey::try_from(public_key)
                                                                .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let p2wpkh_address = Address::p2wpkh(&compressed_public_key, Network::Bitcoin);

    let p2wpkh_scriptpubkey = p2wpkh_address.script_pubkey();

    let version = p2wpkh_scriptpubkey
                                                    .witness_version().ok_or_else(|| LabError::InvalidScript("not a witness script".to_string()))?;

    let program = &p2wpkh_scriptpubkey.as_bytes()[2..];

    let witness_program_report = WitnessProgramReport {
        version: version as u8,
        program_hex: hex::encode(program),
        program_length: program.len()
    };

    Ok(witness_program_report)

    // todo!("Lab 04: inspect the P2WPKH witness program")
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {

    let signature = signature_hex.to_string();
    let public_key= public_key_hex.to_string();

    let native_segwit_spend = NativeSegwitSpend {
        script_sig_hex: "".to_string(),
        witness_items: vec![signature, public_key]
    };

    Ok(native_segwit_spend)

    // todo!("Lab 04: model native P2WPKH unlocking data")
}
