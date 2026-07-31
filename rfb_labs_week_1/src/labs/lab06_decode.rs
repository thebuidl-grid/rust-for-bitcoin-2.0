//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let raw_response = client.call(
        None,
        "getrawtransaction",
        &[txid.to_string(), "2".to_string()],
    )?;

    let transaction: DecodedTransaction = serde_json::from_str(&raw_response)?;
    Ok(transaction)
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
        .find(|out| out.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or_else(|| LabError::MissingField("payment output matching receiver address"))?;

    let change = transaction
        .outputs
        .iter()
        .find(|out| {
            out.address.as_deref() != Some(receiver_address)
                && !out.script_pub_key_hex.starts_with("6a")
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let total_input: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();

    let total_output: f64 = transaction
        .outputs
        .iter()
        .map(|output| output.value)
        .sum();

    let fee = total_input - total_output;

    if fee < 0.0 {
        return Err(LabError::Parse(format!(
            "invalid transaction: negative fee calculated ({fee})"
        )));
    }

    Ok(fee)
}