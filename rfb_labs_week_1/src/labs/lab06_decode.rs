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
    let parsed_response = parse_cli_value(&response)?;

    let inputs = parsed_response
        .get("vin")
        .and_then(|value| value.as_array())
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(|input| {
            let prevout = input
                .get("prevout")
                .ok_or(LabError::MissingField("prevout"))?;
            let vout = u32::try_from(required_u64(input, "vout")?)
                .map_err(|_| LabError::Parse("input vout does not fit in u32".to_owned()))?;

            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: required_string(input, "txid")?,
                    vout,
                },
                previous_value: required_f64(prevout, "value")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let outputs = parsed_response
        .get("vout")
        .and_then(|value| value.as_array())
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|output| {
            let script_pub_key = output
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;
            let vout = u32::try_from(required_u64(output, "n")?)
                .map_err(|_| LabError::Parse("output index does not fit in u32".to_owned()))?;

            Ok(DecodedOutput {
                vout,
                value: required_f64(output, "value")?,
                address: script_pub_key
                    .get("address")
                    .and_then(|value| value.as_str())
                    .map(ToOwned::to_owned),
                script_pub_key_hex: required_string(script_pub_key, "hex")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: required_string(&parsed_response, "txid")?,
        inputs,
        outputs,
        vsize: required_u64(&parsed_response, "vsize")?,
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
        .ok_or_else(|| LabError::Parse("receiver payment output was not found".to_owned()))?;

    let change = transaction
        .outputs
        .iter()
        .find(|output| output.vout != payment.vout && !output.script_pub_key_hex.starts_with("6a"))
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    const SATOSHIS_PER_BTC: f64 = 100_000_000.0;

    let input_total_sats: i64 = transaction
        .inputs
        .iter()
        .map(|input| (input.previous_value * SATOSHIS_PER_BTC).round() as i64)
        .sum();
    let output_total_sats: i64 = transaction
        .outputs
        .iter()
        .map(|output| (output.value * SATOSHIS_PER_BTC).round() as i64)
        .sum();

    if output_total_sats > input_total_sats {
        return Err(LabError::Parse(
            "transaction outputs exceed its inputs".to_owned(),
        ));
    }

    Ok((input_total_sats - output_total_sats) as f64 / SATOSHIS_PER_BTC)
}
