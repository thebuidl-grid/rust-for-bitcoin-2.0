//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use std::str::FromStr;

use bitcoin::{
    hashes::{hash160, Hash},
    opcodes::all::{OP_CHECKMULTISIG, OP_EQUAL, OP_HASH160},
    script::Builder,
    Address, Network, PublicKey, ScriptBuf,
};

use crate::model::P2shReport;
use crate::{LabError, LabResult};

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    let parsed_keys: Vec<PublicKey> = public_keys
        .iter()
        .map(|key| {
            PublicKey::from_str(key).map_err(|error| {
                LabError::InvalidKey(format!("invalid multisig public key `{key}`: {error}"))
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let redeem_script = Builder::new()
        .push_int(2)
        .push_key(&parsed_keys[0])
        .push_key(&parsed_keys[1])
        .push_key(&parsed_keys[2])
        .push_int(3)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(hex::encode(redeem_script.as_bytes()))
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    let redeem_script = parse_script(redeem_script_hex)?;

    Address::p2sh(&redeem_script, network)
        .map(|address| address.to_string())
        .map_err(|error| LabError::InvalidScript(format!("cannot derive P2SH address: {error}")))
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let redeem_script = parse_script(redeem_script_hex)?;
    let script_hash = hash160::Hash::hash(redeem_script.as_bytes());

    let script_pubkey = Builder::new()
        .push_opcode(OP_HASH160)
        .push_slice(script_hash.as_byte_array())
        .push_opcode(OP_EQUAL)
        .into_script();

    Ok(hex::encode(script_pubkey.as_bytes()))
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

fn parse_script(script_hex: &str) -> LabResult<ScriptBuf> {
    let bytes = hex::decode(script_hex)
        .map_err(|error| LabError::InvalidScript(format!("invalid script hex: {error}")))?;

    Ok(ScriptBuf::from_bytes(bytes))
}
