//! Lab 06 — decode a transaction and prove value conservation.

use serde::de::value;

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
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
    let cli_response = parse_cli_value(&call)?;

    let txid = required_string(&cli_response, "txid")?;
    let vsize = required_u64(&cli_response, "vsize")?;

    let vin = cli_response
        .get("vin")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField(&"vin"))?;

    let inputs = vin
        .iter()
        .map(|v| {
            let txid = required_string(v, "txid")?;
            let vout = required_u64(v, "vout")? as u32;

            let prevout = v
                .get("prevout")
                .ok_or_else(|| LabError::MissingField("prevout"))?;
            let prevout_value = required_f64(prevout, "value")?;

            Ok(DecodedInput {
                previous_output: OutPoint { txid, vout },
                previous_value: prevout_value,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    let vout = cli_response
        .get("vout")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField(&"vout"))?;

    let outputs = vout
        .iter()
        .map(|v| {
            let n = required_u64(v, "n")? as u32;
            let value = required_f64(v, "value")?;
            let script_pk = v
                .get("scriptPubKey")
                .ok_or_else(|| LabError::MissingField(&"script_pub_key"))?;

            let spk_hex = required_string(script_pk, "hex")?;
            let spk_addr = if required_string(script_pk, "address").is_ok() {
                Some(required_string(script_pk, "address")?)
            } else {
                None
            };

            Ok(DecodedOutput {
                vout: n,
                value,
                address: spk_addr,
                script_pub_key_hex: spk_hex,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

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
     let mut payment: Option<DecodedOutput> = None;
    let mut change: Option<DecodedOutput> = None;

    for output in &transaction.outputs {
        if let Some(addr) = &output.address {
            if addr == receiver_address {
                payment = Some(output.clone());
            } else {
                change = Some(output.clone());
            }
        }
    }

    let payment = payment
        .ok_or_else(|| LabError::Parse("No output found for receiver address".to_string()))?;
    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
///
/// The result is rounded to the nearest satoshi (8 decimal places) to avoid
/// floating-point accumulation errors.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
     let input_sum: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();

    let output_sum: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = input_sum - output_sum;

    if fee < 0.0 {
        return Err(LabError::Parse(
            "Invalid transaction: negative fee".to_string(),
        ));
    }
    Ok(fee)
}
