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

    let decoded_txid = required_string(&val, "txid")?;
    let vsize = required_u64(&val, "vsize")?;

    // 2. Decode inputs (vin)
    let vin_array = val
        .get("vin")
        .and_then(|v| v.as_array())
        .ok_or(LabError::MissingField("vin"))?;

    let mut inputs = Vec::new();
    for vin in vin_array {
        let prev_txid = required_string(vin, "txid")?;
        let prev_vout = required_u64(vin, "vout")? as u32;

        let prevout = vin
            .get("prevout")
            .ok_or(LabError::MissingField("prevout"))?;
        let previous_value = required_f64(prevout, "value")?;

        inputs.push(DecodedInput {
            previous_output: OutPoint {
                txid: prev_txid,
                vout: prev_vout,
            },
            previous_value,
        });
    }

    // 3. Decode outputs (vout)
    let vout_array = val
        .get("vout")
        .and_then(|v| v.as_array())
        .ok_or(LabError::MissingField("vout"))?;

    let mut outputs = Vec::new();
    for vout in vout_array {
        let n = required_u64(vout, "n")? as u32;
        let value = required_f64(vout, "value")?;

        let script_pub_key = vout
            .get("scriptPubKey")
            .ok_or(LabError::MissingField("scriptPubKey"))?;
        let script_pub_key_hex = required_string(script_pub_key, "hex")?;

        // Address can be in scriptPubKey.address (or scriptPubKey.addresses array in older RPCs)
        let address = script_pub_key
            .get("address")
            .and_then(|a| a.as_str())
            .map(ToOwned::to_owned);

        outputs.push(DecodedOutput {
            vout: n,
            value,
            address,
            script_pub_key_hex,
        });
    }

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
    // TODO: match the receiver address; treat the remaining non-OP_RETURN output as change.
    let mut payment: Option<DecodedOutput> = None;
    let mut change: Option<DecodedOutput> = None;

    for output in &transaction.outputs {
        // Skip OP_RETURN data outputs (script starting with 6a / OP_RETURN)
        if output.script_pub_key_hex.starts_with("6a") {
            continue;
        }

        if let Some(ref addr) = output.address {
            if addr == receiver_address {
                payment = Some(output.clone());
                continue;
            }
        }

        // Remaining non-OP_RETURN output is treated as change
        if change.is_none() {
            change = Some(output.clone());
        }
    }

    let payment = payment.ok_or(LabError::MissingField(
        "payment output matching receiver_address",
    ))?;

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    // TODO: reject impossible negative fees and return the BTC fee.
     let sum_inputs: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let sum_outputs: f64 = transaction.outputs.iter().map(|o| o.value).sum();

    let fee = sum_inputs - sum_outputs;

    // Check for impossible negative fees due to invalid tx data or floating point math
    if fee < 0.0 {
        return Err(LabError::Parse(format!(
            "invalid negative fee calculated: {fee} (inputs: {sum_inputs}, outputs: {sum_outputs})"
        )));
    }

    Ok(fee)
}
