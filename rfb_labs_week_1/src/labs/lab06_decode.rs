//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let response = client.call(
        None,
        "getrawtransaction",
        &[txid.to_owned(), "2".to_owned()],
    )?;
    let value = parse_cli_value(&response)?;
    let inputs = value
        .get("vin")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(|input| {
            let vout = required_u64(input, "vout")?
                .try_into()
                .map_err(|_| LabError::Parse("vout does not fit in u32".to_owned()))?;
            let prevout = input
                .get("prevout")
                .ok_or(LabError::MissingField("prevout"))?;
            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: required_string(input, "txid")?,
                    vout,
                },
                previous_value: required_f64(prevout, "value")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;
    let outputs = value
        .get("vout")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|output| {
            let script = output
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;
            let vout = required_u64(output, "n")?
                .try_into()
                .map_err(|_| LabError::Parse("output index does not fit in u32".to_owned()))?;
            Ok(DecodedOutput {
                vout,
                value: required_f64(output, "value")?,
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
        .ok_or(LabError::MissingField("receiver payment output"))?;
    let change = transaction
        .outputs
        .iter()
        .find(|output| {
            output.vout != payment.vout
                && !output
                    .script_pub_key_hex
                    .to_ascii_lowercase()
                    .starts_with("6a")
        })
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
    // Bitcoin amounts have satoshi precision, so normalize floating-point arithmetic
    // before exposing the result to callers.
    let fee = ((input_total - output_total) * 100_000_000.0).round() / 100_000_000.0;
    if fee < -0.000_000_001 {
        return Err(LabError::Parse(format!(
            "transaction outputs ({output_total}) exceed inputs ({input_total})"
        )));
    }
    Ok(fee.max(0.0))
}
