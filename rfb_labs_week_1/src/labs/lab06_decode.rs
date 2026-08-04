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

    let vin = value
        .get("vin")
        .and_then(|field| field.as_array())
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
        .and_then(|field| field.as_array())
        .ok_or(LabError::MissingField("vout"))?;
    let outputs = vout
        .iter()
        .map(|entry| {
            let script_pub_key = entry
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;
            Ok(DecodedOutput {
                vout: required_u64(entry, "n")? as u32,
                value: required_f64(entry, "value")?,
                address: script_pub_key
                    .get("address")
                    .and_then(|field| field.as_str())
                    .map(str::to_owned),
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
        .ok_or_else(|| LabError::Parse("no output pays the receiver address".to_owned()))?;

    let change = transaction
        .outputs
        .iter()
        .find(|output| output.vout != payment.vout)
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
    // Bitcoin amounts are always whole satoshis (1e-8 BTC); rounding to that
    // precision clears the sub-satoshi noise `f64` subtraction leaves behind.
    let fee = ((input_total - output_total) * 100_000_000.0).round() / 100_000_000.0;

    if fee < 0.0 {
        return Err(LabError::Parse(format!(
            "negative fee: inputs {input_total} < outputs {output_total}"
        )));
    }

    Ok(fee)
}
