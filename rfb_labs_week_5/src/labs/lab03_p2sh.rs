//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use bitcoin::blockdata::opcodes::all::{OP_CHECKMULTISIG, OP_EQUAL, OP_HASH160};
use bitcoin::script::Builder;
use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::model::P2shReport;
use crate::LabResult;

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    let keys = public_keys
        .iter()
        .map(|key| {
            key.parse::<PublicKey>()
                .map_err(|error| crate::LabError::InvalidKey(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

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
    let script = ScriptBuf::from_hex(redeem_script_hex)
        .map_err(|error| crate::LabError::InvalidScript(error.to_string()))?;

    let address = Address::p2sh(&script, network)
        .map_err(|error| crate::LabError::InvalidScript(error.to_string()))?;

    Ok(address.to_string())
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let script = ScriptBuf::from_hex(redeem_script_hex)
        .map_err(|error| crate::LabError::InvalidScript(error.to_string()))?;

    let script_hash = script.script_hash();

    let script_pubkey = Builder::new()
        .push_opcode(OP_HASH160)
        .push_slice(script_hash)
        .push_opcode(OP_EQUAL)
        .into_script();

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
