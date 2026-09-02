//! Lab 04 — construct and explain native SegWit P2WPKH.

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::LabError::InvalidKey;
use crate::{LabError, LabResult};
use bitcoin::hex::DisplayHex;
use bitcoin::psbt::Error;
use bitcoin::Address;
use bitcoin::{CompressedPublicKey, Network, PublicKey};
use std::io::{Cursor, Read};
use std::str::FromStr;

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    // todo!("Lab 04: encode a version-0, 20-byte witness program")
    let pk = match PublicKey::from_str(public_key_hex) {
        Ok(pk) => pk,
        Err(_) => return Err(InvalidKey(public_key_hex.to_string())),
    };

    let compressed_pub = match CompressedPublicKey::try_from(pk) {
        Ok(pk) => pk,
        Err(_) => return Err(InvalidKey(pk.to_string())),
    };

    let address = Address::p2wpkh(&compressed_pub, network);
    Ok(address.to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    // todo!("Lab 04: construct the native SegWit scriptPubKey")
    let pk = match PublicKey::from_str(public_key_hex) {
        Ok(pk) => pk,
        Err(_) => return Err(InvalidKey(public_key_hex.to_string())),
    };

    let compressed_pub = match CompressedPublicKey::try_from(pk) {
        Ok(pk) => pk,
        Err(_) => return Err(InvalidKey(pk.to_string())),
    };

    let address = Address::p2wpkh(&compressed_pub, Network::Regtest);
    Ok(address.script_pubkey().to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    // todo!("Lab 04: inspect the P2WPKH witness program")
    let pk = match PublicKey::from_str(public_key_hex) {
        Ok(pk) => pk,
        Err(_) => return Err(InvalidKey(public_key_hex.to_string())),
    };

    let compressed_pub = match CompressedPublicKey::try_from(pk) {
        Ok(pk) => pk,
        Err(_) => return Err(InvalidKey(pk.to_string())),
    };

    let address = Address::p2wpkh(&compressed_pub, Network::Regtest).script_pubkey();

    let bytes = &address[1..21];
    let program_hex = hex::encode(bytes);

    Ok(WitnessProgramReport {
        version: 0,
        program_hex,
        program_length: bytes.len(),
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    // todo!("Lab 04: model native P2WPKH unlocking data")

    Ok(NativeSegwitSpend {
        script_sig_hex: "".to_string(),
        witness_items: vec![signature_hex.to_string(), public_key_hex.to_string()],
    })
}
