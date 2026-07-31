//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};
use serde::Deserialize;

const SATOSHIS_PER_BTC: f64 = 100_000_000.0;

#[derive(Deserialize)]
struct RpcTransaction {
    txid: String,
    vsize: u64,
    vin: Vec<RpcInput>,
    vout: Vec<RpcOutput>,
}

#[derive(Deserialize)]
struct RpcInput {
    txid: String,
    vout: u32,
    prevout: RpcPreviousOutput,
}

#[derive(Deserialize)]
struct RpcPreviousOutput {
    value: f64,
}

#[derive(Deserialize)]
struct RpcOutput {
    value: f64,
    n: u32,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: RpcScriptPubKey,
}

#[derive(Deserialize)]
struct RpcScriptPubKey {
    hex: String,
    address: Option<String>,
}

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
    let transaction: RpcTransaction = serde_json::from_value(parse_cli_value(&response)?)?;

    Ok(DecodedTransaction {
        txid: transaction.txid,
        inputs: transaction
            .vin
            .into_iter()
            .map(|input| DecodedInput {
                previous_output: OutPoint {
                    txid: input.txid,
                    vout: input.vout,
                },
                previous_value: input.prevout.value,
            })
            .collect(),
        outputs: transaction
            .vout
            .into_iter()
            .map(|output| DecodedOutput {
                vout: output.n,
                value: output.value,
                address: output.script_pub_key.address,
                script_pub_key_hex: output.script_pub_key.hex,
            })
            .collect(),
        vsize: transaction.vsize,
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
        if output.address.as_deref() == Some(receiver_address) {
            if payment.replace(output.clone()).is_some() {
                return Err(LabError::Parse(
                    "transaction has multiple outputs to the receiver address".into(),
                ));
            }
        } else if !output.script_pub_key_hex.starts_with("6a")
            && change.replace(output.clone()).is_some()
        {
            return Err(LabError::Parse(
                "transaction has multiple possible change outputs".into(),
            ));
        }
    }

    Ok(PaymentAndChange {
        payment: payment.ok_or(LabError::MissingField("receiver payment output"))?,
        change,
    })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_satoshis = transaction.inputs.iter().try_fold(0_i64, |sum, input| {
        btc_to_satoshis(input.previous_value).and_then(|value| {
            sum.checked_add(value)
                .ok_or_else(|| LabError::Parse("input value overflow".into()))
        })
    })?;
    let output_satoshis = transaction.outputs.iter().try_fold(0_i64, |sum, output| {
        btc_to_satoshis(output.value).and_then(|value| {
            sum.checked_add(value)
                .ok_or_else(|| LabError::Parse("output value overflow".into()))
        })
    })?;
    let fee_satoshis = input_satoshis
        .checked_sub(output_satoshis)
        .ok_or_else(|| LabError::Parse("transaction output value exceeds input value".into()))?;

    if fee_satoshis < 0 {
        return Err(LabError::Parse(
            "transaction output value exceeds input value".into(),
        ));
    }

    Ok(fee_satoshis as f64 / SATOSHIS_PER_BTC)
}

fn btc_to_satoshis(value: f64) -> LabResult<i64> {
    if !value.is_finite() || value < 0.0 {
        return Err(LabError::Parse("invalid non-negative BTC value".into()));
    }

    let satoshis = value * SATOSHIS_PER_BTC;
    if satoshis > i64::MAX as f64 {
        return Err(LabError::Parse("BTC value exceeds supported range".into()));
    }

    Ok(satoshis.round() as i64)
}
