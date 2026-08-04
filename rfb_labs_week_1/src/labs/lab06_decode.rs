//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{
    DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange,
};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let response = client.call(None, "getrawtransaction", &[txid.to_string(), "2".to_string()])?;
    let value = parse_cli_value(&response)?;

    let mut inputs = Vec::new();
    for vin in value["vin"].as_array().unwrap() {
        inputs.push(DecodedInput {
            previous_output: OutPoint {
                txid: required_string(vin, "txid")?,
                vout: required_u64(vin, "vout")? as u32,
            },
            previous_value: required_f64(&vin["prevout"], "value")?,
        });
    }

    let mut outputs = Vec::new();
    for vout in value["vout"].as_array().unwrap() {
        let script = &vout["scriptPubKey"];

        outputs.push(DecodedOutput {
            vout: required_u64(vout, "n")? as u32,
            value: required_f64(vout, "value")?,
            address: script["address"].as_str().map(String::from),
            script_pub_key_hex: required_string(script, "hex")?,
        });
    }

    Ok(DecodedTransaction {
        txid: required_string(&value, "txid")?,
        inputs,
        outputs,
        vsize: required_u64(&value, "vsize")?,
    })
}

/// Return every previous output consumed by the transaction.
pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
    transaction
        .inputs
        .iter()
        .map(|i| i.previous_output.clone())
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
        .ok_or(LabError::Parse("receiver output not found".into()))?;

    let change = transaction
        .outputs
        .iter()
        .find(|o| {
            o.address.as_deref() != Some(receiver_address)
                && !o.script_pub_key_hex.starts_with("6a")
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_sum: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let output_sum: f64 = transaction.outputs.iter().map(|o| o.value).sum();

    let fee = input_sum - output_sum;

    if fee < 0.0 {
        return Err(LabError::Parse("negative fee".into()));
    }

    Ok(fee)
}