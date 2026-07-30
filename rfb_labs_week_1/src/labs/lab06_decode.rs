//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::RpcClient;
use crate::LabError;
use crate::LabResult;

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    // TODO: call getrawtransaction with verbosity 2 and decode:
    // - txid and vsize
    // - each vin's txid, vout, and prevout.value
    // - each vout's n, value, scriptPubKey.hex, and optional address
    let raw = client.call(
        None,
        "getrawtransaction",
        &[txid.to_owned(), "2".to_owned()],
    )?;
    let value = crate::rpc::parse_cli_value(&raw)?;

    let inputs = value
        .get("vin")
        .and_then(|v| v.as_array())
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(|entry| {
            let previous_output = OutPoint {
                txid: crate::rpc::required_string(entry, "txid")?,
                vout: crate::rpc::required_u64(entry, "vout")? as u32,
            };
            let previous_value = entry
                .get("prevout")
                .and_then(|prevout| prevout.get("value"))
                .and_then(|v| v.as_f64())
                .ok_or(LabError::MissingField("prevout.value"))?;

            Ok(DecodedInput {
                previous_output,
                previous_value,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let outputs = value
        .get("vout")
        .and_then(|v| v.as_array())
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|entry| {
            let script_pub_key = entry
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;

            Ok(DecodedOutput {
                vout: crate::rpc::required_u64(entry, "n")? as u32,
                value: crate::rpc::required_f64(entry, "value")?,
                address: script_pub_key
                    .get("address")
                    .and_then(|v| v.as_str())
                    .map(ToOwned::to_owned),
                script_pub_key_hex: crate::rpc::required_string(script_pub_key, "hex")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: crate::rpc::required_string(&value, "txid")?,
        inputs,
        outputs,
        vsize: crate::rpc::required_u64(&value, "vsize")?,
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
            LabError::Parse(format!(
                "no output pays the receiver address {receiver_address}"
            ))
        })?;

    let change = transaction
        .outputs
        .iter()
        .find(|output| output.vout != payment.vout && !output.script_pub_key_hex.starts_with("6a"))
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    // TODO: reject impossible negative fees and return the BTC fee.
    const SATS_PER_BTC: f64 = 100_000_000.0;

    let input_total_sats: i64 = transaction
        .inputs
        .iter()
        .map(|input| (input.previous_value * SATS_PER_BTC).round() as i64)
        .sum();
    let output_total_sats: i64 = transaction
        .outputs
        .iter()
        .map(|output| (output.value * SATS_PER_BTC).round() as i64)
        .sum();
    let fee_sats = input_total_sats - output_total_sats;

    if fee_sats < 0 {
        return Err(LabError::Parse(format!(
            "calculated a negative fee ({fee_sats} sats), inputs must cover outputs"
        )));
    }

    Ok(fee_sats as f64 / SATS_PER_BTC)
}
