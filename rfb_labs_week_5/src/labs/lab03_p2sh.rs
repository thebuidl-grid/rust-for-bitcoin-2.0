//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::model::P2shReport;
use crate::{LabError, LabResult};

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    let mut builder = Builder::new().push_int(2);

    for key in public_keys {
        let parsed_key = key
            .parse::<PublicKey>()
            .map_err(|err| LabError::InvalidKey(err.to_string()))?;
        builder = builder.push_key(&parsed_key);
    }

    let script = builder
        .push_int(3)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(script.to_hex_string())
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    let redeem_script =
        hex::decode(redeem_script_hex).map_err(|err| LabError::InvalidScript(err.to_string()))?;

    let script = ScriptBuf::from_bytes(redeem_script);

    let address =
        Address::p2sh(&script, network).map_err(|err| LabError::InvalidScript(err.to_string()))?;

    Ok(address.to_string())
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let script_bytes =
        hex::decode(redeem_script_hex).map_err(|err| LabError::InvalidScript(err.to_string()))?;

    let redeem_script = ScriptBuf::from_bytes(script_bytes);

    let script_hash = redeem_script.script_hash();

    let script_pub_key = ScriptBuf::new_p2sh(&script_hash);

    Ok(script_pub_key.to_hex_string())
}

/// Collect the inner script, outer address, and scriptPubKey in one report.
pub fn inspect_p2sh_multisig(public_keys: [&str; 3], network: Network) -> LabResult<P2shReport> {
    let redeem_script = build_2_of_3_redeem_script(public_keys)?;

    let address = derive_p2sh_address(&redeem_script, network)?;

    let script_pub_key = build_p2sh_script_pubkey(&redeem_script)?;

    Ok(P2shReport {
        address,
        redeem_script_hex: redeem_script,
        script_pubkey_hex: script_pub_key,
    })
}
