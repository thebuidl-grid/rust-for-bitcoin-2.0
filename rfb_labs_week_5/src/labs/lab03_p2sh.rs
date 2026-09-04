//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use bitcoin::hashes::Hash;
use bitcoin::key::PublicKey;
use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::opcodes::Opcode;
use bitcoin::script::Builder;
use bitcoin::{Network, ScriptBuf};
use std::str::FromStr;

use crate::error::LabError;
use crate::model::P2shReport;
use crate::LabResult;

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    // Parse all three public keys
    let pks: Result<Vec<PublicKey>, _> = public_keys
        .iter()
        .map(|pk_hex| PublicKey::from_str(pk_hex))
        .collect();

    let pks = pks.map_err(|e| LabError::InvalidKey(e.to_string()))?;

    // Build the multisig script manually: 2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG
    let mut builder = Builder::new().push_int(2);
    for pk in &pks {
        builder = builder.push_key(pk);
    }
    let script = builder
        .push_int(3)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(script.to_hex_string())
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    let redeem_script = ScriptBuf::from_hex(redeem_script_hex)
        .map_err(|e| LabError::InvalidScript(e.to_string()))?;

    let address = bitcoin::Address::p2sh(&redeem_script, network)
        .map_err(|e| LabError::InvalidScript(e.to_string()))?;

    Ok(address.to_string())
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let redeem_script = ScriptBuf::from_hex(redeem_script_hex)
        .map_err(|e| LabError::InvalidScript(e.to_string()))?;

    let script_pubkey = ScriptBuf::new_p2sh(&redeem_script.script_hash());
    Ok(script_pubkey.to_hex_string())
}

/// Collect the inner script, outer address, and scriptPubKey in one report.
pub fn inspect_p2sh_multisig(public_keys: [&str; 3], network: Network) -> LabResult<P2shReport> {
    let redeem_script_hex = build_2_of_3_redeem_script(public_keys)?;
    let address = derive_p2sh_address(&redeem_script_hex, network)?;
    let script_pubkey_hex = build_p2sh_script_pubkey(&redeem_script_hex)?;

    Ok(P2shReport {
        redeem_script_hex,
        address,
        script_pubkey_hex,
    })
}
