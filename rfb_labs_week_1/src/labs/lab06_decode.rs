//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, RpcClient};
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

    let txid_result = value
        .get("txid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
        .ok_or(crate::LabError::MissingField("txid"))?;

    let vsize = value
        .get("vsize")
        .and_then(|v| v.as_u64())
        .ok_or(crate::LabError::MissingField("vsize"))?;

    let inputs = value
        .get("vin")
        .and_then(|v| v.as_array())
        .ok_or(crate::LabError::MissingField("vin"))?
        .iter()
        .map(|vin| {
            let txid = vin
                .get("txid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
                .ok_or(crate::LabError::MissingField("vin.txid"))?;
            let vout = vin
                .get("vout")
                .and_then(|v| v.as_u64())
                .ok_or(crate::LabError::MissingField("vin.vout"))? as u32;
            let previous_value = vin
                .get("prevout")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_f64())
                .ok_or(crate::LabError::MissingField("vin.prevout.value"))?;
            Ok::<DecodedInput, crate::LabError>(DecodedInput {
                previous_output: OutPoint { txid, vout },
                previous_value,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let outputs = value
        .get("vout")
        .and_then(|v| v.as_array())
        .ok_or(crate::LabError::MissingField("vout"))?
        .iter()
        .map(|vout| {
            let vout_index =
                vout.get("n")
                    .and_then(|v| v.as_u64())
                    .ok_or(crate::LabError::MissingField("vout.n"))? as u32;
            let value = vout
                .get("value")
                .and_then(|v| v.as_f64())
                .ok_or(crate::LabError::MissingField("vout.value"))?;
            let script_pub_key_hex = vout
                .get("scriptPubKey")
                .and_then(|v| v.get("hex"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
                .ok_or(crate::LabError::MissingField("vout.scriptPubKey.hex"))?;
            let address = vout
                .get("scriptPubKey")
                .and_then(|v| v.get("address"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned());
            Ok::<DecodedOutput, crate::LabError>(DecodedOutput {
                vout: vout_index,
                value,
                address,
                script_pub_key_hex,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DecodedTransaction {
        txid: txid_result,
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

    for output in &transaction.outputs {
        if let Some(ref addr) = output.address {
            if addr == receiver_address {
                payment = Some(output.clone());
            } else {
                change = Some(output.clone());
            }
        }
    }

    payment
        .map(|p| PaymentAndChange { payment: p, change })
        .ok_or_else(|| crate::LabError::Parse("no output found for receiver address".to_owned()))
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_sum: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let output_sum: f64 = transaction.outputs.iter().map(|o| o.value).sum();
    let fee = input_sum - output_sum;

    if fee < 0.0 {
        return Err(crate::LabError::Parse(
            "negative fee (outputs exceed inputs)".to_owned(),
        ));
    }

    Ok((fee * 1e8).round() / 1e8)
}
