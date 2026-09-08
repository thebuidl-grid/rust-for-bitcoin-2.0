//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput,DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    // TODO: call getrawtransaction with verbosity 2 and decode:
    // - txid and vsize
    // - each vin's txid, vout, and prevout.value
    // - each vout's n, value, scriptPubKey.hex, and optional address
     let raw_info = client.call(
        None,
        "getrawtransaction",
        &[txid.to_string(), "2".to_string()],
    )?;
    let info = parse_cli_value(&raw_info)?;

    let vsize = required_u64(&info, "vsize")?;

    let vin = info
        .get("vin")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vin"))?;

    let inputs = vin
        .iter()
        .map(|entry| {
            let input_txid = required_string(entry, "txid")?;
            let vout = entry
                .get("vout")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as u32)
                .ok_or(LabError::MissingField("vout"))?;
            let prevout = entry
                .get("prevout")
                .ok_or(LabError::MissingField("prevout"))?;
            let previous_value = required_f64(prevout, "value")?;

            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: input_txid,
                    vout,
                },
                previous_value,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let vout_entries = info
        .get("vout")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vout"))?;

    let outputs = vout_entries
        .iter()
        .map(|entry| {
            let n = entry
                .get("n")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as u32)
                .ok_or(LabError::MissingField("n"))?;
            let value = required_f64(entry, "value")?;
            let script_pub_key = entry
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;
            let script_pub_key_hex = required_string(script_pub_key, "hex")?;
            let address = script_pub_key
                .get("address")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned);

            Ok(DecodedOutput {
                vout: n,
                value,
                address,
                script_pub_key_hex,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: required_string(&info, "txid")?,
        inputs,
        outputs,
        vsize,
    })
}

/// Return every previous output consumed by the transaction.
pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
    // TODO: map decoded inputs to their outpoints.
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
    // TODO: match the receiver address; treat the remaining non-OP_RETURN output as change.
     let payment = transaction
        .outputs
        .iter()
        .find(|output| output.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or_else(|| {
            LabError::Rpc(format!(
                "no output pays the expected receiver address {receiver_address}"
            ))
        })?;

    let change = transaction
        .outputs
        .iter()
        .find(|output| {
            output.vout != payment.vout && !output.script_pub_key_hex.starts_with("6a")
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    // TODO: reject impossible negative fees and return the BTC fee.
      let total_in: f64 = transaction.inputs.iter().map(|input| input.previous_value).sum();
    let total_out: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = total_in - total_out;

    if fee < 0.0 {
        return Err(LabError::Rpc(format!(
            "calculated negative fee ({fee}): inputs {total_in} < outputs {total_out}"
        )));
    }

    // Round to satoshi precision (8 decimals) to eliminate f64 rounding noise.
    let rounded_fee = (fee * 100_000_000.0).round() / 100_000_000.0;

    Ok(rounded_fee)

}
