//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};

use crate::labs::lab_helper::required_array;

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
    let value = parse_cli_value(&raw)?;
    let txid_out = required_string(&value, "txid")?;
    let vsize = required_u64(&value, "vsize")?;

    // vin[]: prevout must include value at verbosity=2
    let vin_arr = required_array(&value, "vin")?;

    let inputs = vin_arr
        .iter()
        .map(|vin| {
            // OutPoint: { txid, vout }
            let prev_txid = required_string(vin, "txid")?;
            let prev_vout_u64 = required_u64(vin, "vout")?;
            let prevout = OutPoint {
                txid: prev_txid,
                vout: prev_vout_u64 as u32,
            };

            let previous_value = vin
                .get("prevout")
                .and_then(|p| p.get("value"))
                .and_then(serde_json::Value::as_f64)
                .ok_or(LabError::MissingField("vin.prevout.value"))?;

            Ok(DecodedInput {
                previous_output: prevout,
                previous_value,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    // vout[]: include n, value, scriptPubKey.hex and optional scriptPubKey.address
    let vout_arr = value
        .get("vout")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::Parse("expected vout array".to_string()))?;

    let outputs = vout_arr
        .iter()
        .map(|vout| {
            let vout_n = required_u64(vout, "n")? as u32;
            let value_btc = required_f64(vout, "value")?;

            let spk = vout
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("vout.scriptPubKey"))?;

            let script_pub_key_hex = required_string(spk, "hex")?;

            let address = spk
                .get("address")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned);

            Ok(DecodedOutput {
                vout: vout_n,
                value: value_btc,
                address,
                script_pub_key_hex,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

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
    let mut receiver_output: Option<DecodedOutput> = None;
    let mut change_output: Option<DecodedOutput> = None;

    for output in &transaction.outputs {
        // Skip OP_RETURN outputs (they don't represent spendable payment/change).
        // They normally have no address, so we detect them by scriptPubKey hex.
        if output.script_pub_key_hex.starts_with("6a") {
            continue;
        }

        // If this output matches the receiver address, treat it as the payment.
        if output.address.as_deref() == Some(receiver_address) {
            receiver_output = Some(output.clone());
            continue;
        }

        // Otherwise, if it's not receiver and not OP_RETURN, treat as change.
        // If there are multiple such outputs, we keep the first (or you can error).
        if change_output.is_none() {
            change_output = Some(output.clone());
        }
    }

    let receiver_output = receiver_output.ok_or_else(|| {
        LabError::Parse(format!(
            "receiver output not found for address {receiver_address}"
        ))
    })?;

    Ok(PaymentAndChange {
        payment: receiver_output,
        change: change_output,
    })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    // Convert BTC (f64) -> satoshis with rounding.
    // 1 BTC = 100_000_000 sats
    let to_sats = |btc: f64| -> i64 {
        (btc * 100_000_000.0).round() as i64
    };

    let input_sats: i64 = transaction.inputs.iter().map(|i| to_sats(i.previous_value)).sum();
    let output_sats: i64 = transaction.outputs.iter().map(|o| to_sats(o.value)).sum();

    let fee_sats = input_sats - output_sats;

    if fee_sats < 0 {
        return Err(LabError::Parse(format!(
            "impossible negative fee: {fee_sats} sats"
        )));
    }

    Ok(fee_sats as f64 / 100_000_000.0)
}