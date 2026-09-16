//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use bitcoin::hashes::Hash;
use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::Network;
use bitcoin::PublicKey;
use std::str::FromStr;

use crate::model::P2shReport;
use crate::LabResult;

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    let mut builder = Builder::new();
    builder = builder.push_int(2);

    for key_hex in &public_keys {
        let pubkey =
            PublicKey::from_str(key_hex).map_err(|e| crate::LabError::InvalidKey(e.to_string()))?;
        builder = builder.push_key(&pubkey);
    }

    builder = builder.push_int(3);
    builder = builder.push_opcode(OP_CHECKMULTISIG);

    let script = builder.into_script();
    Ok(script.to_hex_string())
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    let script_bytes = hex::decode(redeem_script_hex)
        .map_err(|e| crate::LabError::InvalidScript(e.to_string()))?;

    let script = bitcoin::ScriptBuf::from_bytes(script_bytes);
    let address = bitcoin::Address::p2sh(&script, network)
        .map_err(|e| crate::LabError::InvalidScript(e.to_string()))?;

    Ok(address.to_string())
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let script_bytes = hex::decode(redeem_script_hex)
        .map_err(|e| crate::LabError::InvalidScript(e.to_string()))?;

    let script = bitcoin::ScriptBuf::from_bytes(script_bytes);
    use bitcoin::hashes::Hash;
    let script_hash = bitcoin::hashes::hash160::Hash::hash(script.as_ref());

    let pubkey_script = Builder::new()
        .push_opcode(bitcoin::opcodes::all::OP_HASH160)
        .push_slice(script_hash.as_byte_array())
        .push_opcode(bitcoin::opcodes::all::OP_EQUAL)
        .into_script();

    Ok(pubkey_script.to_hex_string())
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
