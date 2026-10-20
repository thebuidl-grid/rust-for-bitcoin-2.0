//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::LabError;
use crate::LabResult;

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

    let vin = value
        .get("vin")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vin"))?;
    let inputs = vin
        .iter()
        .map(|entry| {
            let prevout = entry
                .get("prevout")
                .ok_or(LabError::MissingField("prevout"))?;
            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: required_string(entry, "txid")?,
                    vout: required_u64(entry, "vout")? as u32,
                },
                previous_value: required_f64(prevout, "value")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let vout = value
        .get("vout")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vout"))?;
    let outputs = vout
        .iter()
        .map(|entry| {
            let script = entry
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;
            Ok(DecodedOutput {
                vout: required_u64(entry, "n")? as u32,
                value: required_f64(entry, "value")?,
                address: script
                    .get("address")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
                script_pub_key_hex: required_string(script, "hex")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: required_string(&value, "txid")?,
        inputs,
        outputs,
        vsize: required_u64(&value, "vsize")?,
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

fn is_op_return_output(output: &DecodedOutput) -> bool {
    output.script_pub_key_hex.starts_with("6a") || output.script_pub_key_hex.starts_with("6A")
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
        .ok_or_else(|| {
            LabError::Parse(format!(
                "no output found for receiver address {receiver_address}"
            ))
        })?;

    let change = transaction
        .outputs
        .iter()
        .filter(|output| output.vout != payment.vout && !is_op_return_output(output))
        .max_by_key(|output| output.vout)
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_total: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();
    let output_total: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = input_total - output_total;

    if fee < 0.0 {
        return Err(LabError::Parse(
            "transaction outputs exceed input value".to_owned(),
        ));
    }

    Ok((fee * 100_000_000.0).round() / 100_000_000.0)
}
