//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let response = client.call(None, "getrawtransaction", &[txid.to_owned(), "2".to_string()])?;
    let value = parse_cli_value(&response)?;

    let inputs = value
        .get("vin")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(|entry| {
            let previous_output = OutPoint {
                txid: entry
                    .get("txid")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::MissingField("txid"))?,
                vout: entry
                    .get("vout")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(LabError::MissingField("vout"))? as u32,
            };

            let previous_value = entry
                .get("prevout")
                .and_then(|prevout| prevout.get("value"))
                .and_then(serde_json::Value::as_f64)
                .ok_or(LabError::MissingField("prevout.value"))?;

            Ok(DecodedInput {
                previous_output,
                previous_value,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let outputs = value
        .get("vout")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|entry| {
            let script_pub_key = entry
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;

            Ok(DecodedOutput {
                vout: entry
                    .get("n")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(LabError::MissingField("n"))? as u32,
                value: entry
                    .get("value")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or(LabError::MissingField("value"))?,
                address: script_pub_key
                    .get("address")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
                script_pub_key_hex: script_pub_key
                    .get("hex")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::MissingField("scriptPubKey.hex"))?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: value
            .get("txid")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(LabError::MissingField("txid"))?,
        inputs,
        outputs,
        vsize: value
            .get("vsize")
            .and_then(serde_json::Value::as_u64)
            .ok_or(LabError::MissingField("vsize"))?,
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
    let payment = transaction
        .outputs
        .iter()
        .find(|output| output.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or(LabError::Parse(
            "receiver output not found in decoded transaction".to_string(),
        ))?;

    let change = transaction
        .outputs
        .iter()
        .filter(|output| output.vout != payment.vout)
        .find(|output| {
            output.address.is_some() && !output.script_pub_key_hex.to_ascii_lowercase().starts_with("6a")
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_sum: f64 = transaction.inputs.iter().map(|input| input.previous_value).sum();
    let output_sum: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = input_sum - output_sum;

    if fee < 0.0 {
        return Err(LabError::Parse(
            "transaction outputs exceed inputs; negative fee".to_string(),
        ));
    }

    Ok((fee * 100_000_000.0).round() / 100_000_000.0)
}
