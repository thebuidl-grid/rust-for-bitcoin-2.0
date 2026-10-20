//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let raw = client.call(
        None,
        "getrawtransaction",
        &[txid.to_owned(), "2".to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;
    let txid_out = required_string(&value, "txid")?;
    let vsize = required_u64(&value, "vsize")?;

    let vin_arr = value
        .get("vin")
        .and_then(Value::as_array)
        .ok_or_else(|| LabError::MissingField("vin"))?;
    let mut inputs = Vec::new();
    for vin in vin_arr {
        let prev_txid = required_string(vin, "txid")?;
        let prev_vout = required_u64(vin, "vout")? as u32;
        let prevout = vin
            .get("prevout")
            .ok_or_else(|| LabError::MissingField("prevout"))?;
        let prev_value = required_f64(prevout, "value")?;
        inputs.push(DecodedInput {
            previous_output: OutPoint {
                txid: prev_txid,
                vout: prev_vout,
            },
            previous_value: prev_value,
        });
    }

    let vout_arr = value
        .get("vout")
        .and_then(Value::as_array)
        .ok_or_else(|| LabError::MissingField("vout"))?;
    let mut outputs = Vec::new();
    for vout in vout_arr {
        let n = required_u64(vout, "n")? as u32;
        let value = required_f64(vout, "value")?;
        let script_pub_key = vout
            .get("scriptPubKey")
            .ok_or_else(|| LabError::MissingField("scriptPubKey"))?;
        let script_pub_key_hex = required_string(script_pub_key, "hex")?;
        let address = script_pub_key
            .get("address")
            .and_then(Value::as_str)
            .map(String::from);
        outputs.push(DecodedOutput {
            vout: n,
            value,
            address,
            script_pub_key_hex,
        });
    }

    Ok(DecodedTransaction {
        txid: txid_out,
        inputs,
        outputs,
        vsize,
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
    let mut payment = None;
    let mut change = None;
    for out in &transaction.outputs {
        if out.address.as_deref() == Some(receiver_address) {
            payment = Some(out.clone());
        } else if !out.script_pub_key_hex.starts_with("6a") {
            change = Some(out.clone());
        }
    }
    let payment = payment.ok_or_else(|| {
        LabError::Rpc(format!(
            "Receiver address {} not found in outputs",
            receiver_address
        ))
    })?;
    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let sum_inputs: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();
    let sum_outputs: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = sum_inputs - sum_outputs;
    if fee < 0.0 {
        return Err(LabError::Rpc("Negative fee calculated".to_owned()));
    }
    let rounded_fee = (fee * 100_000_000.0).round() / 100_000_000.0;
    Ok(rounded_fee)
}
