//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Satoshis in one bitcoin.
const SATOSHIS_PER_BTC: f64 = 100_000_000.0;

/// Convert a BTC amount into whole satoshis.
///
/// Consensus counts satoshis, which are integers, but `f64` cannot represent most
/// BTC amounts exactly. Totalling in satoshis keeps the arithmetic exact so a fee
/// of one thousand satoshis never reads back as `0.000009999999`.
fn to_satoshis(btc: f64) -> i64 {
    (btc * SATOSHIS_PER_BTC).round() as i64
}

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    // Verbosity 2 is what makes each input carry its `prevout`, and without the
    // consumed values there is no way to derive the fee.
    let raw = client.call(
        None,
        "getrawtransaction",
        &[txid.to_owned(), "2".to_owned()],
    )?;
    let transaction = parse_cli_value(&raw)?;

    let inputs = transaction
        .get("vin")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vin"))?
        .iter()
        .map(decode_input)
        .collect::<LabResult<Vec<_>>>()?;

    let outputs = transaction
        .get("vout")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(decode_output)
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: required_string(&transaction, "txid")?,
        inputs,
        outputs,
        vsize: required_u64(&transaction, "vsize")?,
    })
}

/// Decode one input into the outpoint it consumes and the value it carried.
fn decode_input(vin: &Value) -> LabResult<DecodedInput> {
    let prevout = vin
        .get("prevout")
        .ok_or(LabError::MissingField("prevout"))?;

    Ok(DecodedInput {
        previous_output: OutPoint {
            txid: required_string(vin, "txid")?,
            vout: decode_index(vin, "vout")?,
        },
        previous_value: required_f64(prevout, "value")?,
    })
}

/// Decode one output, including its locking script and optional address.
fn decode_output(vout: &Value) -> LabResult<DecodedOutput> {
    let script = vout
        .get("scriptPubKey")
        .ok_or(LabError::MissingField("scriptPubKey"))?;

    Ok(DecodedOutput {
        // Core calls the output index `n` here, not `vout`.
        vout: decode_index(vout, "n")?,
        value: required_f64(vout, "value")?,
        // Absent for scripts with no standard address form, such as OP_RETURN.
        address: script
            .get("address")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        script_pub_key_hex: required_string(script, "hex")?,
    })
}

/// Read an output index, which the model stores as a `u32`.
fn decode_index(value: &Value, field: &'static str) -> LabResult<u32> {
    u32::try_from(required_u64(value, field)?)
        .map_err(|_| LabError::Parse(format!("`{field}` does not fit in a u32")))
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
                "transaction {} has no output paying {receiver_address}",
                transaction.txid
            ))
        })?;

    // Whatever is left over, other than a data carrier, returns to the sender.
    // A spend that consumed its inputs exactly would have no change at all.
    let change = transaction
        .outputs
        .iter()
        .find(|output| output.vout != payment.vout && !is_op_return(output))
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// An OP_RETURN output begins with opcode `0x6a` and is provably unspendable.
fn is_op_return(output: &DecodedOutput) -> bool {
    output.script_pub_key_hex.starts_with("6a")
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let consumed: i64 = transaction
        .inputs
        .iter()
        .map(|input| to_satoshis(input.previous_value))
        .sum();

    let assigned: i64 = transaction
        .outputs
        .iter()
        .map(|output| to_satoshis(output.value))
        .sum();

    // Creating value out of nothing is exactly what consensus forbids.
    if assigned > consumed {
        return Err(LabError::Parse(format!(
            "transaction {} assigns {assigned} satoshis but only consumes {consumed}",
            transaction.txid
        )));
    }

    Ok((consumed - assigned) as f64 / SATOSHIS_PER_BTC)
}
