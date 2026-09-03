//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let call = client.call(
        None,
        "getrawtransaction",
        &[txid.to_string(), "2".to_string()],
    )?;
    let response = parse_cli_value(&call)?;

    let txid = required_string(&response, "txid")?;
    let vsize = response
        .get("vsize")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| LabError::MissingField("vsize"))?;

    let vin = response
        .get("vin")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("vin"))?;

    let inputs: Vec<DecodedInput> = vin
        .iter()
        .map(|input| {
            let in_txid = required_string(input, "txid")?;
            let vout = input
                .get("vout")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| LabError::MissingField("vout"))? as u32;

            let prevout = input
                .get("prevout")
                .ok_or_else(|| LabError::MissingField("prevout"))?;
            let previous_value = required_f64(prevout, "value")?;

            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: in_txid,
                    vout,
                },
                previous_value,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let vout = response
        .get("vout")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("vout"))?;

    let outputs: Vec<DecodedOutput> = vout
        .iter()
        .map(|output| {
            let n = output
                .get("n")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| LabError::MissingField("n"))? as u32;
            let value = required_f64(output, "value")?;

            let script_pub_key = output
                .get("scriptPubKey")
                .ok_or_else(|| LabError::MissingField("scriptPubKey"))?;
            let script_pub_key_hex = required_string(script_pub_key, "hex")?;
            let address = script_pub_key
                .get("address")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(DecodedOutput {
                vout: n,
                value,
                address,
                script_pub_key_hex,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;
    Ok(DecodedTransaction {
        txid,
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
    let mut payment: Option<DecodedOutput> = None;
    let mut change: Option<DecodedOutput> = None;

    for output in &transaction.outputs {
        if let Some(addr) = &output.address {
            if addr == receiver_address {
                payment = Some(output.clone());
            } else {
                change = Some(output.clone());
            }
        }
    }

    let payment = payment
        .ok_or_else(|| LabError::Parse("No output found for receiver address".to_string()))?;
    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_sum: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();

    let output_sum: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = input_sum - output_sum;

    if fee < 0.0 {
        return Err(LabError::Parse(
            "Invalid transaction: negative fee".to_string(),
        ));
    }
    Ok(fee)
}
