//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Verbosity level that makes Bitcoin Core attach each input's `prevout`.
const PREVOUT_VERBOSITY: &str = "2";

/// Satoshis in one BTC. Bitcoin amounts are whole satoshis, so the audit converts
/// Bitcoin Core's BTC-denominated JSON before doing any arithmetic on it.
const SATOSHIS_PER_BTC: f64 = 100_000_000.0;

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    // Verbosity 2 is what makes the audit possible: the raw transaction itself never
    // states how much its inputs are worth, only which outputs they point at.
    let raw = client.call(
        None,
        "getrawtransaction",
        &[txid.to_owned(), PREVOUT_VERBOSITY.to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;

    let inputs = array_field(&value, "vin")?
        .iter()
        .map(|entry| decode_input(client, entry))
        .collect::<LabResult<Vec<_>>>()?;
    let outputs = array_field(&value, "vout")?
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
                "no output of {} pays `{receiver_address}`",
                transaction.txid
            ))
        })?;

    // Whatever the sender did not pay away and did not leave to the miner comes back
    // to a fresh address the sending wallet controls.
    let change = transaction
        .outputs
        .iter()
        .find(|output| output.vout != payment.vout && !is_op_return(output))
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    // Summing BTC as floating point accumulates rounding error, which would show up as
    // a fee that is a fraction of a satoshi away from the real one. Satoshis are exact.
    let input_total: i64 = transaction
        .inputs
        .iter()
        .map(|input| to_satoshis(input.previous_value))
        .sum();
    let output_total: i64 = transaction
        .outputs
        .iter()
        .map(|output| to_satoshis(output.value))
        .sum();
    let fee = input_total - output_total;

    // A transaction that creates more value than it consumes is invalid by consensus,
    // so anything below zero here is a decoding mistake rather than a cheap fee.
    if fee < 0 {
        return Err(LabError::Parse(format!(
            "{} spends {input_total} sat but creates {output_total} sat",
            transaction.txid
        )));
    }

    Ok(fee as f64 / SATOSHIS_PER_BTC)
}

/// Convert a BTC amount from Bitcoin Core's JSON into whole satoshis.
fn to_satoshis(amount_btc: f64) -> i64 {
    (amount_btc * SATOSHIS_PER_BTC).round() as i64
}

/// Borrow a named array field, such as `vin` or `vout`.
fn array_field<'a>(value: &'a Value, field: &'static str) -> LabResult<&'a Vec<Value>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField(field))
}

/// Decode one `vin` entry together with the value of the output it consumes.
fn decode_input<C: RpcClient>(client: &C, entry: &Value) -> LabResult<DecodedInput> {
    let previous_output = OutPoint {
        txid: required_string(entry, "txid")?,
        vout: required_u64(entry, "vout")? as u32,
    };

    // Bitcoin Core reads `prevout` from a block's undo data, so verbosity 2 only
    // carries it once the transaction has been mined. While the spend is still sitting
    // in the mempool the consumed output is untouched in the confirmed UTXO set, which
    // is exactly where `gettxout` can still price it.
    let previous_value = match entry.get("prevout") {
        Some(prevout) => required_f64(prevout, "value")?,
        None => previous_output_value(client, &previous_output)?,
    };

    Ok(DecodedInput {
        previous_output,
        previous_value,
    })
}

/// Look up what an outpoint is worth in the confirmed UTXO set.
fn previous_output_value<C: RpcClient>(client: &C, point: &OutPoint) -> LabResult<f64> {
    // `include_mempool = false` is what makes the output visible at all: the very
    // transaction being decoded spends it, so a mempool-aware view reports it as gone.
    let raw = client.call(
        None,
        "gettxout",
        &[
            point.txid.clone(),
            point.vout.to_string(),
            "false".to_owned(),
        ],
    )?;

    required_f64(&parse_cli_value(&raw)?, "value").map_err(|_| {
        LabError::Parse(format!(
            "could not price the output {}:{} spent by this transaction",
            point.txid, point.vout
        ))
    })
}

/// Decode one `vout` entry.
fn decode_output(entry: &Value) -> LabResult<DecodedOutput> {
    let script_pub_key = entry
        .get("scriptPubKey")
        .ok_or(LabError::MissingField("scriptPubKey"))?;

    Ok(DecodedOutput {
        vout: required_u64(entry, "n")? as u32,
        value: required_f64(entry, "value")?,
        address: script_pub_key
            .get("address")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        script_pub_key_hex: required_string(script_pub_key, "hex")?,
    })
}

/// `OP_RETURN` outputs are provably unspendable, so they can never hold change.
fn is_op_return(output: &DecodedOutput) -> bool {
    output.script_pub_key_hex.starts_with("6a")
}
