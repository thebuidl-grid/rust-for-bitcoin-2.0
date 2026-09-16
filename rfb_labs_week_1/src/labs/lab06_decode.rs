//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};
use serde::Deserialize;

#[derive(Deserialize)]
struct RawVin {
    txid: String,
    vout: u32,
    prevout: RawPrevout,
}

#[derive(Deserialize)]
struct RawPrevout {
    value: f64,
}

#[derive(Deserialize)]
struct RawVout {
    n: u32,
    value: f64,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: RawScriptPubKey,
}

#[derive(Deserialize)]
struct RawScriptPubKey {
    hex: String,
    address: Option<String>,
}

#[derive(Deserialize)]
struct RawDecodedTx {
    txid: String,
    vsize: u64,
    vin: Vec<RawVin>,
    vout: Vec<RawVout>,
}

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    // TODO: call getrawtransaction with verbosity 2 and decode:
    // - txid and vsize
    // - each vin's txid, vout, and prevout.value
    // - each vout's n, value, scriptPubKey.hex, and optional address
    // todo!("Lab 06: decode a verbose raw transaction")
    let call = client.call(
        None,
        "getrawtransaction",
        &[txid.to_string(), "2".to_string()],
    )?;

    let parsed: RawDecodedTx =
        serde_json::from_str(&call).map_err(|e| LabError::Parse(e.to_string()))?;

    let inputs = parsed
        .vin
        .into_iter()
        .map(|vin| DecodedInput {
            previous_output: OutPoint {
                txid: vin.txid,
                vout: vin.vout,
            },
            previous_value: vin.prevout.value,
        })
        .collect();

    let outputs = parsed
        .vout
        .into_iter()
        .map(|vout| DecodedOutput {
            vout: vout.n,
            value: vout.value,
            address: vout.script_pub_key.address,
            script_pub_key_hex: vout.script_pub_key.hex,
        })
        .collect();

    Ok(DecodedTransaction {
        txid: parsed.txid,
        inputs,
        outputs,
        vsize: parsed.vsize,
    })
}

/// Return every previous output consumed by the transaction.
pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
    // TODO: map decoded inputs to their outpoints.
    // todo!("Lab 06: list consumed outpoints")
    transaction
        .inputs
        .iter()
        .map(|input| input.previous_output.clone())
        .collect()
}

/// Identify the receiver payment and optional change output.
pub fn identify_payment_and_change(
    transaction: &DecodedTransaction,
    receiver_address: &str,
) -> LabResult<PaymentAndChange> {
    // TODO: match the receiver address; treat the remaining non-OP_RETURN output as change.
    // todo!("Lab 06: identify payment and change")
    let payment = transaction
        .outputs
        .iter()
        .find(|out| out.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or(LabError::MissingField("payment output to receiver_address"))?;

    let change = transaction
        .outputs
        .iter()
        .find(|out| {
            out.address.as_deref() != Some(receiver_address)
                && !out.script_pub_key_hex.starts_with("6a") // OP_RETURN opcode in hex
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    // TODO: reject impossible negative fees and return the BTC fee.
    // todo!("Lab 06: calculate the miner fee")
    let total_in: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let total_out: f64 = transaction.outputs.iter().map(|o| o.value).sum();

    let raw_fee = total_in - total_out;

    // Round to satoshi precision (8 decimal places) to eliminate floating-point drift.
    let fee = (raw_fee * 100_000_000.0).round() / 100_000_000.0;

    if fee < 0.0 {
        return Err(LabError::Parse(format!(
            "calculated negative fee: inputs={} outputs={} fee={}",
            total_in, total_out, fee
        )));
    }

    Ok(fee)
}
