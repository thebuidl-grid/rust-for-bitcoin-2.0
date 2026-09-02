//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use std::str::FromStr;

use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::model::P2shReport;
use crate::{LabError, LabResult};

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    let keys = [
        parse_public_key(public_keys[0])?,
        parse_public_key(public_keys[1])?,
        parse_public_key(public_keys[2])?,
    ];

    let redeem_script = Builder::new()
        .push_int(2)
        .push_key(&keys[0])
        .push_key(&keys[1])
        .push_key(&keys[2])
        .push_int(3)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(redeem_script.to_hex_string())
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    let redeem_script = parse_script(redeem_script_hex)?;

    Address::p2sh(&redeem_script, network)
        .map(|address| address.to_string())
        .map_err(|error| LabError::InvalidScript(error.to_string()))
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let redeem_script = parse_script(redeem_script_hex)?;

    // The outer script commits to the hash only. The 2-of-3 rule is invisible here
    // and stays invisible on-chain until an input reveals the redeemScript.
    Ok(ScriptBuf::new_p2sh(&redeem_script.script_hash()).to_hex_string())
}

/// Collect the inner script, outer address, and scriptPubKey in one report.
pub fn inspect_p2sh_multisig(public_keys: [&str; 3], network: Network) -> LabResult<P2shReport> {
    let redeem_script_hex = build_2_of_3_redeem_script(public_keys)?;

    Ok(P2shReport {
        address: derive_p2sh_address(&redeem_script_hex, network)?,
        script_pubkey_hex: build_p2sh_script_pubkey(&redeem_script_hex)?,
        redeem_script_hex,
    })
}

/// Parse a compressed or uncompressed public key from hex.
fn parse_public_key(public_key_hex: &str) -> LabResult<PublicKey> {
    PublicKey::from_str(public_key_hex.trim())
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Parse a serialized script from hex.
fn parse_script(script_hex: &str) -> LabResult<ScriptBuf> {
    ScriptBuf::from_hex(script_hex.trim())
        .map_err(|error| LabError::InvalidScript(error.to_string()))
}
