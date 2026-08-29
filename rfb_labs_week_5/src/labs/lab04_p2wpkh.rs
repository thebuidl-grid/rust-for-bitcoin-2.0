//! Lab 04 — construct and explain native SegWit P2WPKH.

use std::vec;

use bitcoin::{Address, CompressedPublicKey, Network, PublicKey, ScriptBuf};

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::{LabError, LabResult};

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;

    let compressed_key = CompressedPublicKey::try_from(public_key)
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;

    let address = Address::p2wpkh(&compressed_key, network);

    Ok(address.to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    // get the public key
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;

    // get the compressed key
    let compressed_key = CompressedPublicKey::try_from(public_key)
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;

    // get the public key hash

    let public_key_hash = compressed_key.wpubkey_hash();

    // extract the script pub key
    let script_public_key = ScriptBuf::new_p2wpkh(&public_key_hash);

    Ok(script_public_key.to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;

    let compressed_key = CompressedPublicKey::try_from(public_key)
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;

    let program_hex = compressed_key.wpubkey_hash().to_string();

    let program_length = program_hex.len() / 2;

    Ok(WitnessProgramReport {
        version: 0,
        program_hex,
        program_length,
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    //  vlaidate the signature hex

    hex::decode(signature_hex).map_err(|err| LabError::InvalidScript(err.to_string()))?;

    //  get the public key
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;

    // get the compressed key

    let compressed_key = CompressedPublicKey::try_from(public_key)
        .map_err(|err| LabError::InvalidKey(err.to_string()))?;

    // return the native spend object
    Ok(NativeSegwitSpend {
        script_sig_hex: "".to_string(),
        witness_items: vec![signature_hex.to_owned(), compressed_key.to_string()],
    })
}
