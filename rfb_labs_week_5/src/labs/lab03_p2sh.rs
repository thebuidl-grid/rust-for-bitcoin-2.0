//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use crate::model::P2shReport;
use crate::LabError::InvalidKey;
use crate::{LabError, LabResult};
use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::{Address, Network, PublicKey, ScriptBuf};
use std::str::FromStr;

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    // todo!("Lab 03: build a canonical 2-of-3 multisig redeemScript")
    let keys = public_keys
        .iter()
        .map(|key| PublicKey::from_str(key).map_err(|_| InvalidKey(key.to_string())))
        .collect::<Result<Vec<_>, _>>()?;

    let scriptbuf = Builder::new()
        .push_int(2)
        .push_key(&keys[0])
        .push_key(&keys[1])
        .push_key(&keys[2])
        .push_int(3)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(scriptbuf.to_hex_string())
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    // todo!("Lab 03: HASH160 the redeemScript and encode its P2SH address")
    let bytes =
        hex::decode(redeem_script_hex).map_err(|e| LabError::InvalidScript(e.to_string()))?;

    let script = ScriptBuf::from_bytes(bytes);
    let address =
        Address::p2sh(&script, network).map_err(|e| LabError::InvalidScript(e.to_string()))?;
    Ok(address.to_string())
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    // todo!("Lab 03: build the P2SH outer locking script")
    let bytes =
        hex::decode(redeem_script_hex).map_err(|e| LabError::InvalidScript(e.to_string()))?;

    let script = ScriptBuf::from_bytes(bytes);
    let address = Address::p2sh(&script, Network::Regtest);

    match address {
        Ok(address) => Ok(address.script_pubkey().to_hex_string()),
        Err(e) => Err(LabError::InvalidAddress(e.to_string())),
    }
}

/// Collect the inner script, outer address, and scriptPubKey in one report.
pub fn inspect_p2sh_multisig(public_keys: [&str; 3], network: Network) -> LabResult<P2shReport> {
    // todo!("Lab 03: connect the redeemScript to its P2SH commitment")
    let redeem_script_hex = build_2_of_3_redeem_script(public_keys)?;
    let address = derive_p2sh_address(&redeem_script_hex, network)?;
    let script_pubkey_hex = build_p2sh_script_pubkey(&redeem_script_hex)?;

    Ok(P2shReport {
        redeem_script_hex,
        address,
        script_pubkey_hex,
    })
}
