//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

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

    let inputs = value
        .get("vin")
        .and_then(|vin| vin.as_array())
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(|input| {
            let prevout = input
                .get("prevout")
                .ok_or(LabError::MissingField("prevout"))?;
            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: required_string(input, "txid")?,
                    vout: required_u64(input, "vout")? as u32,
                },
                previous_value: required_f64(prevout, "value")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let outputs = value
        .get("vout")
        .and_then(|vout| vout.as_array())
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|output| {
            let script_pub_key = output
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;
            Ok(DecodedOutput {
                vout: required_u64(output, "n")? as u32,
                value: required_f64(output, "value")?,
                address: script_pub_key
                    .get("address")
                    .and_then(|address| address.as_str())
                    .map(ToOwned::to_owned),
                script_pub_key_hex: required_string(script_pub_key, "hex")?,
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
        .ok_or(LabError::Parse(format!(
            "transaction does not pay receiver address {receiver_address}"
        )))?;

    let change = transaction
        .outputs
        .iter()
        .find(|output| {
            output.vout != payment.vout
                && output.address.is_some()
                && !output.script_pub_key_hex.starts_with("6a")
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
    let fee = input_total - output_total;

    if fee < -0.000_000_001 {
        return Err(LabError::Parse(format!(
            "transaction outputs exceed inputs by {} BTC",
            -fee
        )));
    }

    Ok((fee.max(0.0) * 100_000_000.0).round() / 100_000_000.0)
}
