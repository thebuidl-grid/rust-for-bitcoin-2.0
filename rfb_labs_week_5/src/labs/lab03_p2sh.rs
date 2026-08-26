//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::model::P2shReport;
use crate::{LabError, LabResult};

fn parse_public_key(public_key_hex: &str) -> LabResult<PublicKey> {
    public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

fn parse_redeem_script(redeem_script_hex: &str) -> LabResult<ScriptBuf> {
    ScriptBuf::from_hex(redeem_script_hex)
        .map_err(|error| LabError::InvalidScript(error.to_string()))
}

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    let keys = [
        parse_public_key(public_keys[0])?,
        parse_public_key(public_keys[1])?,
        parse_public_key(public_keys[2])?,
    ];

    let script = Builder::new()
        .push_int(2)
        .push_key(&keys[0])
        .push_key(&keys[1])
        .push_key(&keys[2])
        .push_int(3)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(script.to_hex_string())
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    let redeem_script = parse_redeem_script(redeem_script_hex)?;
    let address = Address::p2sh(&redeem_script, network)
        .map_err(|error| LabError::InvalidScript(error.to_string()))?;
    Ok(address.to_string())
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
///
/// Matching this outer scriptPubKey only proves the spender supplied *some* script
/// hashing to the same value — it says nothing about whether that script's own rules
/// (here, 2-of-3 `OP_CHECKMULTISIG`) are satisfied. That is checked separately, after
/// the redeemScript itself executes.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let redeem_script = parse_redeem_script(redeem_script_hex)?;
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
