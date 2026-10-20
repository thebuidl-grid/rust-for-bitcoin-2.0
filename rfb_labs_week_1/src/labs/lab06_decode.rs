//! Lab 06 — decode a transaction and prove value conservation.
//Author: Yankho Ngolleka - Github: codaMW

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{required_f64, required_string, required_u64, RpcClient};
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
        &[txid.to_owned(), "2".to_owned()],
    )?;
    let value = crate::rpc::parse_cli_value(&raw)?;

    let decoded_txid = required_string(&value, "txid")?;
    let vsize = required_u64(&value, "vsize")?;

    let vin = value
        .get("vin")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vin"))?;
    let inputs = vin
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

    let vout = value
        .get("vout")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vout"))?;
    let outputs = vout
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
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                script_pub_key_hex: required_string(script_pub_key, "hex")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: decoded_txid,
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
    let payment = transaction
        .outputs
        .iter()
        .find(|output| output.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or(LabError::MissingField("payment output"))?;

    // The remaining output is change, as long as it isn't an OP_RETURN (0x6a) output.
    let change = transaction
        .outputs
        .iter()
        .find(|output| {
            output.address.as_deref() != Some(receiver_address)
                && !output.script_pub_key_hex.starts_with("6a")
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let total_in: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();
    let total_out: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = total_in - total_out;

    if fee < 0.0 {
        return Err(LabError::Rpc(format!("impossible negative fee: {fee}")));
    }

    // Bitcoin amounts are exact to the satoshi (1e-8 BTC). Round here to eliminate
    // f64 subtraction drift (e.g. 1.0000000000065512e-5 instead of 1e-5) so exact
    // equality assertions on downstream structs (like MultiUtxoAudit) hold.
    let satoshis = (fee * 100_000_000.0).round();
    Ok(satoshis / 100_000_000.0)
}
