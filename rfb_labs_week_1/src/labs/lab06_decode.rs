//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Satoshis in one bitcoin. Bitcoin amounts are integer satoshis, so BTC arithmetic is
/// done in satoshis to keep sums free of binary floating-point drift.
const SATOSHIS_PER_BTC: f64 = 100_000_000.0;

/// An OP_RETURN locking script begins with opcode `0x6a` and burns its value.
const OP_RETURN_PREFIX: &str = "6a";

fn to_satoshis(btc: f64) -> i64 {
    (btc * SATOSHIS_PER_BTC).round() as i64
}

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    // Verbosity 2 adds each input's `prevout`, which is what makes the fee visible.
    let raw = client.call(
        None,
        "getrawtransaction",
        &[txid.to_owned(), "2".to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;

    let inputs = value
        .get("vin")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(decode_input)
        .collect::<LabResult<Vec<_>>>()?;

    let outputs = value
        .get("vout")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(decode_output)
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: required_string(&value, "txid")?,
        inputs,
        outputs,
        vsize: required_u64(&value, "vsize")?,
    })
}

fn decode_input(vin: &Value) -> LabResult<DecodedInput> {
    let prevout = vin
        .get("prevout")
        .ok_or(LabError::MissingField("prevout"))?;

    Ok(DecodedInput {
        previous_output: OutPoint {
            txid: required_string(vin, "txid")?,
            vout: decode_vout_index(vin, "vout")?,
        },
        previous_value: required_f64(prevout, "value")?,
    })
}

fn decode_output(vout: &Value) -> LabResult<DecodedOutput> {
    let script = vout
        .get("scriptPubKey")
        .ok_or(LabError::MissingField("scriptPubKey"))?;

    Ok(DecodedOutput {
        vout: decode_vout_index(vout, "n")?,
        value: required_f64(vout, "value")?,
        // Absent for scripts with no standard address form, such as OP_RETURN.
        address: script
            .get("address")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        script_pub_key_hex: required_string(script, "hex")?,
    })
}

fn decode_vout_index(value: &Value, field: &'static str) -> LabResult<u32> {
    u32::try_from(required_u64(value, field)?).map_err(|error| LabError::Parse(error.to_string()))
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
        .ok_or_else(|| {
            LabError::Parse(format!(
                "no output of {} pays {receiver_address}",
                transaction.txid
            ))
        })?;

    // Whatever value the payment did not claim returns to the sender as change.
    let change = transaction
        .outputs
        .iter()
        .find(|output| {
            output.vout != payment.vout && !output.script_pub_key_hex.starts_with(OP_RETURN_PREFIX)
        })
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let inputs: i64 = transaction
        .inputs
        .iter()
        .map(|input| to_satoshis(input.previous_value))
        .sum();
    let outputs: i64 = transaction
        .outputs
        .iter()
        .map(|output| to_satoshis(output.value))
        .sum();

    let fee = inputs - outputs;
    if fee < 0 {
        return Err(LabError::Parse(format!(
            "{} spends {} more satoshis than it consumes",
            transaction.txid, -fee
        )));
    }

    Ok(fee as f64 / SATOSHIS_PER_BTC)
}
