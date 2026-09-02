//! Lab 04 — construct and explain native SegWit P2WPKH.

use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::LabResult;

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let compressed = CompressedPublicKey::try_from(public_key)
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(Address::p2wpkh(&compressed, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let compressed = CompressedPublicKey::try_from(public_key)
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(Address::p2wpkh(&compressed, Network::Regtest)
        .script_pubkey()
        .to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let compressed = CompressedPublicKey::try_from(public_key)
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let hash = compressed.wpubkey_hash();

    Ok(WitnessProgramReport {
        version: 0,
        program_hex: hash.to_string(),
        program_length: 20,
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    if signature_hex.is_empty() {
        return Err(crate::LabError::InvalidScript(
            "signature cannot be empty".to_string(),
        ));
    }

    public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(NativeSegwitSpend {
        script_sig_hex: String::new(),
        witness_items: vec![signature_hex.to_owned(), public_key_hex.to_owned()],
    })
}
