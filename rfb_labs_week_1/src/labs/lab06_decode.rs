//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let response = client.call(
        None,
        "getrawtransaction",
        &[txid.to_string(), "2".to_string()],
    )?;
    let transaction_json: serde_json::Value = serde_json::from_str(&response)?;

    let txid = required_string(&transaction_json, "txid")?;
    let vsize = required_u64(&transaction_json, "vsize")?;
    let input_values = transaction_json
        .get("vin")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vin"))?;
    let output_values = transaction_json
        .get("vout")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vout"))?;

    let mut inputs = Vec::with_capacity(input_values.len());
    for input_value in input_values {
        let previous_txid = required_string(input_value, "txid")?;
        let previous_vout = required_u32(input_value, "vout")?;
        let previous_value = match input_value.get("prevout") {
            Some(previous_output) => required_f64(previous_output, "value")?,
            None => {
                // Some Bitcoin Core versions omit `prevout` for an unconfirmed
                // verbosity-2 transaction. Query the chain UTXO view while
                // ignoring the mempool spend to recover the consumed value.
                let response = client.call(
                    None,
                    "gettxout",
                    &[
                        previous_txid.clone(),
                        previous_vout.to_string(),
                        "false".to_string(),
                    ],
                )?;
                let previous_output: serde_json::Value = serde_json::from_str(&response)?;
                required_f64(&previous_output, "value")?
            }
        };

        inputs.push(DecodedInput {
            previous_output: OutPoint {
                txid: previous_txid,
                vout: previous_vout,
            },
            previous_value,
        });
    }

    let mut outputs = Vec::with_capacity(output_values.len());
    for output_value in output_values {
        let value = required_f64(output_value, "value")?;
        let vout = required_u32(output_value, "n")?;
        let script = output_value
            .get("scriptPubKey")
            .ok_or(LabError::MissingField("scriptPubKey"))?;
        let script_pub_key_hex = required_string(script, "hex")?;
        let address = optional_string(script, "address")?;

        outputs.push(DecodedOutput {
            vout,
            value,
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
    let mut outpoints = Vec::with_capacity(transaction.inputs.len());
    for input in &transaction.inputs {
        outpoints.push(input.previous_output.clone());
    }

    outpoints
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
        .ok_or_else(|| LabError::Parse("receiver payment output was not found".to_string()))?;

    let change = transaction
        .outputs
        .iter()
        .find(|output| {
            output.vout != payment.vout
                && !output
                    .script_pub_key_hex
                    .to_ascii_lowercase()
                    .starts_with("6a")
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let total_input_value: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();
    let total_output_value: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = total_input_value - total_output_value;

    if fee < 0.0 {
        return Err(LabError::Parse(
            "transaction outputs exceed its inputs".to_string(),
        ));
    }

    // Bitcoin Core reports BTC with eight decimal places. Rounding through
    // satoshis prevents tiny floating-point artifacts in the lab result.
    let fee_satoshis = (fee * 100_000_000.0).round();
    Ok(fee_satoshis / 100_000_000.0)
}

fn required_u32(value: &serde_json::Value, field: &'static str) -> LabResult<u32> {
    let number = required_u64(value, field)?;
    u32::try_from(number).map_err(|_| LabError::Parse(format!("`{field}` does not fit in u32")))
}

fn optional_string(value: &serde_json::Value, field: &'static str) -> LabResult<Option<String>> {
    let Some(field_value) = value.get(field) else {
        return Ok(None);
    };
    let field_value = field_value
        .as_str()
        .ok_or_else(|| LabError::Parse(format!("invalid `{field}` field")))?;

    Ok(Some(field_value.to_owned()))
}
