//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
use serde::Deserialize;

#[derive(Deserialize)]
struct RawVin {
    txid: String,
    vout: u32,
    prevout: Option<RawPrevout>,
}

#[derive(Deserialize)]
struct RawPrevout {
    value: f64,
}

#[derive(Deserialize)]
struct RawScriptPubKey {
    hex: String,
    address: Option<String>,
}

#[derive(Deserialize)]
struct RawVout {
    value: f64,
    n: u32,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: RawScriptPubKey,
}

#[derive(Deserialize)]
struct RawDecodedTx {
    txid: String,
    vin: Vec<RawVin>,
    vout: Vec<RawVout>,
    vsize: u64,
}

use crate::model::DecodedInput;
use crate::model::DecodedOutput;

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let raw = client.call(None, "getrawtransaction", &[txid.to_owned(), "2".to_owned()])?;
    let json = parse_cli_value(&raw)?;
    let raw_tx: RawDecodedTx = serde_json::from_value(json)?;

    let inputs = raw_tx.vin.into_iter().map(|vin| {
        let val = vin.prevout.map(|p| p.value).unwrap_or(0.0);
        DecodedInput {
            previous_output: OutPoint {
                txid: vin.txid,
                vout: vin.vout,
            },
            previous_value: val,
        }
    }).collect();

    let outputs = raw_tx.vout.into_iter().map(|vout| {
        DecodedOutput {
            vout: vout.n,
            value: vout.value,
            address: vout.script_pub_key.address,
            script_pub_key_hex: vout.script_pub_key.hex,
        }
    }).collect();

    Ok(DecodedTransaction {
        txid: raw_tx.txid,
        inputs,
        outputs,
        vsize: raw_tx.vsize,
    })
}

/// Return every previous output consumed by the transaction.
pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
    transaction.inputs.iter().map(|i| i.previous_output.clone()).collect()
}

/// Identify the receiver payment and optional change output.
pub fn identify_payment_and_change(
    transaction: &DecodedTransaction,
    receiver_address: &str,
) -> LabResult<PaymentAndChange> {
    let payment = transaction.outputs.iter()
        .find(|output| output.address.as_deref() == Some(receiver_address))
        .ok_or_else(|| LabError::Parse("Receiver output not found".to_owned()))?
        .clone();

    let change = transaction.outputs.iter()
        .find(|output| {
            output.address.as_deref() != Some(receiver_address)
            && !output.script_pub_key_hex.starts_with("6a")
        })
        .cloned();

    Ok(PaymentAndChange {
        payment,
        change,
    })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let sum_inputs: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let sum_outputs: f64 = transaction.outputs.iter().map(|o| o.value).sum();
    let fee = sum_inputs - sum_outputs;
    if fee < 0.0 {
        return Err(LabError::Parse("Negative fee calculated".to_owned()));
    }
    // Round to 8 decimal places (Satoshi precision) to avoid floating point issues.
    let rounded_fee = (fee * 100_000_000.0).round() / 100_000_000.0;
    Ok(rounded_fee)
}
