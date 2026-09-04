//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, RpcClient};
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
        &[txid.to_string(), "2".to_string()],
    )?;
    let value = parse_cli_value(&raw)?;

    let inputs = value
        .get("vin")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(|vin| {
            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: vin
                        .get("txid")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .ok_or(LabError::MissingField("txid"))?,
                    vout: vin
                        .get("vout")
                        .and_then(Value::as_u64)
                        .map(|n| n as u32)
                        .ok_or(LabError::MissingField("vout"))?,
                },
                previous_value: vin
                    .get("prevout")
                    .and_then(Value::as_object)
                    .and_then(|prevout| prevout.get("value"))
                    .and_then(Value::as_f64)
                    .ok_or(LabError::MissingField("prevout.value"))?,
            })
        })
        .collect::<LabResult<Vec<DecodedInput>>>()?;

    let outputs = value
        .get("vout")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|vout| {
            Ok(DecodedOutput {
                vout: vout
                    .get("n")
                    .and_then(Value::as_u64)
                    .map(|n| n as u32)
                    .ok_or(LabError::MissingField("n"))?,
                value: vout
                    .get("value")
                    .and_then(Value::as_f64)
                    .ok_or(LabError::MissingField("value"))?,
                address: vout
                    .get("scriptPubKey")
                    .and_then(Value::as_object)
                    .and_then(|script| script.get("address"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                script_pub_key_hex: vout
                    .get("scriptPubKey")
                    .and_then(Value::as_object)
                    .and_then(|script| script.get("hex"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::MissingField("scriptPubKey.hex"))?,
            })
        })
        .collect::<LabResult<Vec<DecodedOutput>>>()?;

    Ok(DecodedTransaction {
        txid: value
            .get("txid")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(LabError::MissingField("txid"))?,
        vsize: value
            .get("vsize")
            .and_then(Value::as_u64)
            .ok_or(LabError::MissingField("vsize"))?,
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
    let mut payment: Option<DecodedOutput> = None;
    let mut change: Option<DecodedOutput> = None;

    for output in &transaction.outputs {
        if output.address.as_deref() == Some(receiver_address) {
            if payment.is_none() {
                payment = Some(output.clone());
                continue;
            }
        }

        if output.address.is_some() {
            change = Some(output.clone());
        }
    }

    let payment = payment.ok_or(LabError::MissingField("payment output"))?;

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
        return Err(LabError::Parse("negative fee".to_owned()));
    }

    // Normalize tiny binary-floating-point errors for Bitcoin values.
    let normalized_fee = (fee * 1e8).round() / 1e8;
    Ok(normalized_fee)
}
