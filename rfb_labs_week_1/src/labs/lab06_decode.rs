//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

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
    let val = parse_cli_value(&raw)?;

    let txid_out = required_string(&val, "txid")?;
    let vsize = required_u64(&val, "vsize")?;

    let vin_arr = val
        .get("vin")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("vin"))?;

    let mut inputs = Vec::with_capacity(vin_arr.len());
    for input_item in vin_arr {
        let prev_txid = required_string(input_item, "txid")?;
        let prev_vout = required_u64(input_item, "vout")? as u32;
        let prev_value = input_item
            .get("prevout")
            .and_then(|po| po.get("value"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| LabError::MissingField("prevout.value"))?;

        inputs.push(DecodedInput {
            previous_output: OutPoint {
                txid: prev_txid,
                vout: prev_vout,
            },
            previous_value: prev_value,
        });
    }

    let vout_arr = val
        .get("vout")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("vout"))?;

    let mut outputs = Vec::with_capacity(vout_arr.len());
    for output_item in vout_arr {
        let vout_n = output_item
            .get("n")
            .or_else(|| output_item.get("vout"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LabError::MissingField("n"))? as u32;

        let value = required_f64(output_item, "value")?;

        let spk = output_item
            .get("scriptPubKey")
            .ok_or_else(|| LabError::MissingField("scriptPubKey"))?;

        let address = spk
            .get("address")
            .and_then(|a| a.as_str())
            .map(String::from);
        let script_pub_key_hex = required_string(spk, "hex")?;

        outputs.push(DecodedOutput {
            vout: vout_n,
            value,
            address,
            script_pub_key_hex,
        });
    }

    Ok(DecodedTransaction {
        txid: txid_out,
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
        .find(|o| o.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or_else(|| LabError::Parse("receiver output not found".to_string()))?;

    let change = transaction
        .outputs
        .iter()
        .find(|o| o.vout != payment.vout && !o.script_pub_key_hex.starts_with("6a"))
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let sum_in: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let sum_out: f64 = transaction.outputs.iter().map(|o| o.value).sum();
    let fee = sum_in - sum_out;

    if fee < 0.0 {
        return Err(LabError::Parse("negative fee".to_string()));
    }

    let rounded_fee = (fee * 100_000_000.0).round() / 100_000_000.0;
    Ok(rounded_fee)
}
