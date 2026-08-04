//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_string, required_u64, RpcClient};
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
    let val = parse_cli_value(&call)?;

    let txid = required_string(&val, "txid")?;
    let vsize = required_u64(&val, "vsize")?;

    let vin_array = val
        .get("vin")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("vin"))?;

    let mut inputs = Vec::new();
    for item in vin_array {
        let input_txid = required_string(item, "txid")?;
        let vout = required_u64(item, "vout")? as u32;
        let previous_value = item
            .get("prevout")
            .and_then(|p| p.get("value"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| LabError::MissingField("prevout.value"))?;

        inputs.push(DecodedInput {
            previous_output: OutPoint {
                txid: input_txid,
                vout,
            },
            previous_value,
        });
    }

    let vout_array = val
        .get("vout")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("vout"))?;

    let mut outputs = Vec::new();
    for item in vout_array {
        let vout = required_u64(item, "n")? as u32;
        let value = item
            .get("value")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| LabError::MissingField("value"))?;

        let script_pub_key_hex = item
            .get("scriptPubKey")
            .and_then(|s| s.get("hex"))
            .and_then(|h| h.as_str())
            .unwrap_or_default()
            .to_string();

        let address = item
            .get("scriptPubKey")
            .and_then(|s| s.get("address"))
            .and_then(|a| a.as_str())
            .map(String::from);

        outputs.push(DecodedOutput {
            vout,
            value,
            script_pub_key_hex,
            address,
        });
    }

    Ok(DecodedTransaction {
        txid,
        vsize,
        inputs,
        outputs,
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
        .ok_or_else(|| LabError::Parse("no output pays the receiver address".to_string()))?;

    let change = transaction
        .outputs
        .iter()
        .find(|o| o.address.as_deref() != Some(receiver_address) && o.address.is_some())
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let total_in: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let total_out: f64 = transaction.outputs.iter().map(|o| o.value).sum();

    let fee = total_in - total_out;
    if fee < -1e-8 {
        return Err(LabError::Parse("calculated fee is negative".to_string()));
    }

    let rounded_fee = (fee * 100_000_000.0).round() / 100_000_000.0;
    Ok(rounded_fee)
}
