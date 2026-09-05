//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    // 1. Call `getrawtransaction` with txid and verbosity 2 (or true for verbose JSON with prevout)
    let raw = client.call(None, "getrawtransaction", &[txid.into(), 2.to_string()])?;
    let value = parse_cli_value(&raw)?;

    let returned_txid = required_string(&value, "txid")?;
    let vsize = value["vsize"]
        .as_u64()
        .ok_or_else(|| LabError::Parse("getrawtransaction missing 'vsize'".to_owned()))?;

    // 2. Decode inputs (vin)
    let vin_array = value["vin"]
        .as_array()
        .ok_or_else(|| LabError::Parse("getrawtransaction missing 'vin' array".to_owned()))?;

    let mut inputs = Vec::with_capacity(vin_array.len());
    for vin in vin_array {
        let input_txid = required_string(vin, "txid")?;
        let input_vout = vin["vout"]
            .as_u64()
            .ok_or_else(|| LabError::Parse("vin missing 'vout'".to_owned()))? as u32;

        let prevout = vin
            .get("prevout")
            .ok_or_else(|| LabError::Parse("vin missing 'prevout' object (ensure verbosity=2)".to_owned()))?;

        let input_value = required_f64(prevout, "value")?;

        inputs.push(DecodedInput {
            previous_output: OutPoint {
                txid: input_txid,
                vout: input_vout,
            },
            previous_value: input_value,
        });
    }

    // 3. Decode outputs (vout)
    let vout_array = value["vout"]
        .as_array()
        .ok_or_else(|| LabError::Parse("getrawtransaction missing 'vout' array".to_owned()))?;

    let mut outputs = Vec::with_capacity(vout_array.len());
    for vout in vout_array {
        let n = vout["n"]
            .as_u64()
            .ok_or_else(|| LabError::Parse("vout missing 'n'".to_owned()))? as u32;

        let value_btc = required_f64(vout, "value")?;

        let script_pub_key = vout
            .get("scriptPubKey")
            .ok_or_else(|| LabError::Parse("vout missing 'scriptPubKey'".to_owned()))?;

        let script_hex = required_string(script_pub_key, "hex")?;

        // Extract address if present (can be single string or inside 'address'/'addresses' field)
        let address = script_pub_key
            .get("address")
            .and_then(|a| a.as_str())
            .map(ToOwned::to_owned);

        outputs.push(DecodedOutput {
            vout: n,
            value: value_btc,
            script_pub_key_hex: script_hex,
            address,
        });
    }

    Ok(DecodedTransaction {
        txid: returned_txid,
        vsize,
        inputs,
        outputs,
    })
}

/// Return every previous output consumed by the transaction.
pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
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
    let mut payment_output = None;
    let mut change_output = None;

    for output in &transaction.outputs {
        // Skip OP_RETURN data outputs (script starts with 6a / OP_RETURN)
        if output.script_pub_key_hex.starts_with("6a") {
            continue;
        }

        if let Some(ref addr) = output.address {
            if addr == receiver_address {
                payment_output = Some(output.clone());
            } else {
                change_output = Some(output.clone());
            }
        } else {
            // Non-receiver output without explicit address field is treated as change
            change_output = Some(output.clone());
        }
    }

    let payment = payment_output
        .ok_or_else(|| LabError::Parse("could not find output matching receiver address".to_owned()))?;

    Ok(PaymentAndChange {
        payment,
        change: change_output,
    })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let sum_inputs: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let sum_outputs: f64 = transaction.outputs.iter().map(|o| o.value).sum();

    // Round to satoshi precision (1e-8 BTC) to avoid floating-point summation drift.
    let fee = ((sum_inputs - sum_outputs) * 1e8).round() / 1e8;

    // Reject impossible negative fees (sum_outputs > sum_inputs)
    if fee < 0.0 {
        return Err(LabError::Parse(format!(
            "invalid negative fee calculated: {fee} BTC (inputs: {sum_inputs}, outputs: {sum_outputs})"
        )));
    }

    Ok(fee)
}