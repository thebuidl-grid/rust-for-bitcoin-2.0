//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;
/// Decode a transaction with verbosity 2.
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
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(|vin| {
            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: vin
                        .get("txid")
                        .and_then(Value::as_str)
                        .ok_or(LabError::MissingField("vin.txid"))?
                        .to_owned(),
                    vout: vin
                        .get("vout")
                        .and_then(Value::as_u64)
                        .ok_or(LabError::MissingField("vin.vout"))?
                        as u32,
                },
                previous_value: vin
                    .get("prevout")
                    .and_then(|v| v.get("value"))
                    .and_then(Value::as_f64)
                    .ok_or(LabError::MissingField("prevout.value"))?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let outputs = value
        .get("vout")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|output| {
            let script = output
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;

            Ok(DecodedOutput {
                vout: output
                    .get("n")
                    .and_then(Value::as_u64)
                    .ok_or(LabError::MissingField("vout.n"))? as u32,

                value: output
                    .get("value")
                    .and_then(Value::as_f64)
                    .ok_or(LabError::MissingField("vout.value"))?,

                address: script
                    .get("address")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),

                script_pub_key_hex: script
                    .get("hex")
                    .and_then(Value::as_str)
                    .ok_or(LabError::MissingField("scriptPubKey.hex"))?
                    .to_owned(),
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: value
            .get("txid")
            .and_then(Value::as_str)
            .ok_or(LabError::MissingField("txid"))?
            .to_owned(),

        inputs,
        outputs,

        vsize: value
            .get("vsize")
            .and_then(Value::as_u64)
            .ok_or(LabError::MissingField("vsize"))?,
    })
}

/// Return all consumed outpoints.
pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
    transaction
        .inputs
        .iter()
        .map(|input| input.previous_output.clone())
        .collect()
}

/// Find receiver payment and optional change output.
pub fn identify_payment_and_change(
    transaction: &DecodedTransaction,
    receiver_address: &str,
) -> LabResult<PaymentAndChange> {
    let payment = transaction
        .outputs
        .iter()
        .find(|output| output.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or(LabError::MissingField("payment output"))?;

    let change = transaction
        .outputs
        .iter()
        .find(|output| {
            output.address.is_some() && output.address.as_deref() != Some(receiver_address)
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate miner fee: inputs - outputs.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_total = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum::<f64>();

    let output_total = transaction
        .outputs
        .iter()
        .map(|output| output.value)
        .sum::<f64>();

    let fee = input_total - output_total;

    if fee < 0.0 {
        return Err(LabError::Parse("Transaction has negative fee".to_owned()));
    }

    Ok(fee)
}
