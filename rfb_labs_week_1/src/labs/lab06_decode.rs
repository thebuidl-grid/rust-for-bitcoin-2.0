//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::RpcClient;
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
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    let inputs = value["vin"]
        .as_array()
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(|vin| {
            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: vin["txid"]
                        .as_str()
                        .ok_or(LabError::MissingField("vin.txid"))?
                        .to_string(),
                    vout: vin["vout"]
                        .as_u64()
                        .ok_or(LabError::MissingField("vin.vout"))?
                        as u32,
                },
                previous_value: vin["prevout"]["value"]
                    .as_f64()
                    .ok_or(LabError::MissingField("vin.prevout.value"))?,
            })
        })
        .collect::<LabResult<Vec<DecodedInput>>>()?;

    let outputs = value["vout"]
        .as_array()
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|vout| {
            Ok(DecodedOutput {
                vout: vout["n"].as_u64().ok_or(LabError::MissingField("vout.n"))? as u32,
                value: vout["value"]
                    .as_f64()
                    .ok_or(LabError::MissingField("vout.value"))?,
                address: vout["scriptPubKey"]["address"]
                    .as_str()
                    .map(ToOwned::to_owned),
                script_pub_key_hex: vout["scriptPubKey"]["hex"]
                    .as_str()
                    .ok_or(LabError::MissingField("vout.scriptPubKey.hex"))?
                    .to_string(),
            })
        })
        .collect::<LabResult<Vec<DecodedOutput>>>()?;

    Ok(DecodedTransaction {
        txid: value["txid"]
            .as_str()
            .ok_or(LabError::MissingField("txid"))?
            .to_string(),
        inputs,
        outputs,
        vsize: value["vsize"]
            .as_u64()
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
    let payment_index = transaction
        .outputs
        .iter()
        .position(|output| output.address.as_deref() == Some(receiver_address))
        .ok_or(LabError::MissingField("payment output"))?;

    let change = transaction
        .outputs
        .iter()
        .enumerate()
        .find(|(index, output)| {
            *index != payment_index && !output.script_pub_key_hex.starts_with("6a")
        })
        .map(|(_, output)| output.clone());

    Ok(PaymentAndChange {
        payment: transaction.outputs[payment_index].clone(),
        change,
    })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_sum: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();
    let output_sum: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    // Round to satoshi precision (1e-8 BTC) to remove binary floating-point noise
    // from summing many amounts, matching Bitcoin Core's own fixed-point accounting.
    let fee = ((input_sum - output_sum) * 100_000_000.0).round() / 100_000_000.0;

    if fee < 0.0 {
        return Err(LabError::Parse(format!(
            "computed negative fee: inputs={input_sum}, outputs={output_sum}"
        )));
    }

    Ok(fee)
}
