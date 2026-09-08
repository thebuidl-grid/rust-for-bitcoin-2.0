//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use std::ops::{Add, Index};
use std::str::FromStr;

use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::{Network, PublicKey, Address, ScriptBuf};

use crate::model::P2shReport;
use crate::LabResult;
use crate::LabError;

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {

    let pub_key1_hex = public_keys[0];
    let pub_key2_hex = public_keys[1];
    let pub_key3_hex = public_keys[2];

    let pub_key1 = PublicKey::from_str(pub_key1_hex)
    .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let pub_key2 = PublicKey::from_str(pub_key2_hex)
    .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let pub_key3 = PublicKey::from_str(pub_key3_hex)
    .map_err(|e| LabError::InvalidKey(e.to_string()))?;

    let p2sh_script = Builder::new()
                        .push_int(2)
                        .push_key(&pub_key1)
                        .push_key(&pub_key2)
                        .push_key(&pub_key3)
                        .push_int(3)
                        .push_opcode(OP_CHECKMULTISIG)
                        .into_script();

    // let p2sh_address = Address::p2sh(&p2sh_script, Network::Bitcoin)
    // .map_err(|e| LabError::InvalidScript(e.to_string()))?;

    Ok(p2sh_script.to_hex_string())

    // todo!("Lab 03: build a canonical 2-of-3 multisig redeemScript")
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {

    let bytes = hex::decode(redeem_script_hex)
                                                .map_err(|e| LabError::InvalidScript(e.to_string()))?;

    let script = ScriptBuf::from_bytes(bytes);

    let p2sh_address = Address::p2sh(&script, network)
                                                    .map_err(|e| LabError::InvalidScript(e.to_string()))?;

    Ok(p2sh_address.to_string())

    // todo!("Lab 03: HASH160 the redeemScript and encode its P2SH address")

}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {

    let bytes = hex::decode(redeem_script_hex)
                                                .map_err(|e| LabError::InvalidScript(e.to_string()))?;

    let script = ScriptBuf::from_bytes(bytes);

    let p2sh_address = Address::p2sh(&script, Network::Bitcoin)
                                                    .map_err(|e| LabError::InvalidScript(e.to_string()))?;

    let script_pub_key = p2sh_address.script_pubkey();

    Ok(script_pub_key.to_hex_string())
    // todo!("Lab 03: build the P2SH outer locking script")
}

/// Collect the inner script, outer address, and scriptPubKey in one report.
pub fn inspect_p2sh_multisig(public_keys: [&str; 3], network: Network) -> LabResult<P2shReport> {

    let redeem_script = build_2_of_3_redeem_script(public_keys)?;
    let p2sh_address = derive_p2sh_address(&redeem_script, network)?;
    let p2sh_scriptpubkey = build_p2sh_script_pubkey(&redeem_script)?;

    let p2sh_report = P2shReport {
        redeem_script_hex: redeem_script,
        address: p2sh_address,
        script_pubkey_hex: p2sh_scriptpubkey
    };

    Ok(p2sh_report)

    // todo!("Lab 03: connect the redeemScript to its P2SH commitment")
}
