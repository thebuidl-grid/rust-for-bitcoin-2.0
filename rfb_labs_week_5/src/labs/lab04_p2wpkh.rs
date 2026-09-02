//! Lab 04 — construct and explain native SegWit P2WPKH.

use bitcoin::hashes::Hash;
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};
use std::str::FromStr;

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::{LabError, LabResult};

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key =
        PublicKey::from_str(public_key_hex).map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    let compressed = CompressedPublicKey::try_from(public_key)
        .map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    Ok(Address::p2wpkh(&compressed, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let public_key =
        PublicKey::from_str(public_key_hex).map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    let compressed = CompressedPublicKey::try_from(public_key)
        .map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    Ok(Address::p2wpkh(&compressed, Network::Regtest)
        .script_pubkey()
        .to_hex_string())
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
        .witness_version()
        .ok_or_else(|| LabError::InvalidScript("not a witness script".to_string()))?;

    let program = &p2wpkh_scriptpubkey.as_bytes()[2..];

    let witness_program_report = WitnessProgramReport {
        version: version as u8,
        program_hex: hex::encode(program),
        program_length: program.len(),
    };
    Ok(witness_program_report)
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    let sig_bytes = hex::decode(signature_hex).map_err(|e| LabError::InvalidKey(e.to_string()))?;
    let pubkey_bytes =
        hex::decode(public_key_hex).map_err(|e| LabError::InvalidKey(e.to_string()))?;

    // In native SegWit P2WPKH, the scriptSig is empty.
    // The witness contains: [signature, public_key]
    Ok(NativeSegwitSpend {
        script_sig_hex: "".to_string(),
        witness_items: vec![hex::encode(sig_bytes), hex::encode(pubkey_bytes)],
    })
}
