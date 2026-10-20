//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::RpcClient;
use crate::LabResult;

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

    let txid = crate::rpc::required_string(&value, "txid")?;
    let vsize = crate::rpc::required_u64(&value, "vsize")?;

    let vin_array = value
        .get("vin")
        .and_then(serde_json::Value::as_array)
        .ok_or(crate::LabError::MissingField("vin"))?;

    let mut inputs = Vec::new();
    for item in vin_array {
        let prev_txid = crate::rpc::required_string(item, "txid")?;
        let prev_vout = crate::rpc::required_u64(item, "vout")? as u32;
        let prev_out = item
            .get("prevout")
            .ok_or(crate::LabError::MissingField("prevout"))?;
        let prev_value = crate::rpc::required_f64(prev_out, "value")?;

        inputs.push(crate::model::DecodedInput {
            previous_output: OutPoint {
                txid: prev_txid,
                vout: prev_vout,
            },
            previous_value: prev_value,
        });
    }

    let vout_array = value
        .get("vout")
        .and_then(serde_json::Value::as_array)
        .ok_or(crate::LabError::MissingField("vout"))?;

    let mut outputs = Vec::new();
    for item in vout_array {
        let n = crate::rpc::required_u64(item, "n")? as u32;
        let value_btc = crate::rpc::required_f64(item, "value")?;
        let spk = item
            .get("scriptPubKey")
            .ok_or(crate::LabError::MissingField("scriptPubKey"))?;
        let script_pub_key_hex = crate::rpc::required_string(spk, "hex")?;
        let address = spk
            .get("address")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);

        outputs.push(crate::model::DecodedOutput {
            vout: n,
            value: value_btc,
            address,
            script_pub_key_hex,
        });
    }

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
    let payment = transaction
        .outputs
        .iter()
        .find(|output| output.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or(crate::LabError::MissingField("payment output"))?;

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
    let total_inputs: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let total_outputs: f64 = transaction.outputs.iter().map(|o| o.value).sum();

    let fee = total_inputs - total_outputs;
    if fee < 0.0 {
        return Err(crate::LabError::Rpc("negative fee".to_owned()));
    }

    let fee_rounded = (fee * 100_000_000.0).round() / 100_000_000.0;
    Ok(fee_rounded)
}
