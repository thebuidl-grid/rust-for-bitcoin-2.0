//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::model::P2shReport;
use crate::LabResult;

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    let key1 = public_keys[0]
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let key2 = public_keys[1]
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let key3 = public_keys[2]
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let script = Builder::new()
        .push_int(2)
        .push_key(&key1)
        .push_key(&key2)
        .push_key(&key3)
        .push_int(3)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(script.to_hex_string())
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    let bytes = hex::decode(redeem_script_hex)
        .map_err(|error| crate::LabError::InvalidScript(error.to_string()))?;

    let redeem_script = ScriptBuf::from_bytes(bytes);

    let address = Address::p2sh(&redeem_script, network)
        .map_err(|error| crate::LabError::InvalidScript(error.to_string()))?;

    Ok(address.to_string())
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let bytes = hex::decode(redeem_script_hex)
        .map_err(|error| crate::LabError::InvalidScript(error.to_string()))?;

    let redeem_script = ScriptBuf::from_bytes(bytes);

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
