use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let raw = client.call(
        None,
        "getrawtransaction",
        &[txid.to_owned(), "2".to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;

    let vin_entries = value
        .get("vin")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vin"))?;

    let mut inputs = Vec::with_capacity(vin_entries.len());
    for entry in vin_entries {
        let previous_output = OutPoint {
            txid: required_string(entry, "txid")?,
            vout: required_u64(entry, "vout")? as u32,
        };

        let previous_value = match entry.get("prevout") {
            Some(prevout) => required_f64(prevout, "value")?,
            None => {
                // This node doesn't embed prevout on verbosity 2 — look it up directly.
                let raw_txout = client.call(
                    None,
                    "gettxout",
                    &[
                        previous_output.txid.clone(),
                        previous_output.vout.to_string(),
                        "false".to_owned(),
                    ],
                )?;
                let txout_value = parse_cli_value(&raw_txout)?;
                required_f64(&txout_value, "value")?
            }
        };

        inputs.push(DecodedInput {
            previous_output,
            previous_value,
        });
    }

    let outputs = value
        .get("vout")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("vout"))?
        .iter()
        .map(|entry| {
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
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(DecodedTransaction {
        txid: required_string(&value, "txid")?,
        inputs,
        outputs,
        vsize: required_u64(&value, "vsize")?,
    })
}

pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
    transaction
        .inputs
        .iter()
        .map(|input| input.previous_output.clone())
        .collect()
}

pub fn identify_payment_and_change(
    transaction: &DecodedTransaction,
    receiver_address: &str,
) -> LabResult<PaymentAndChange> {
    let payment = transaction
        .outputs
        .iter()
        .find(|output| output.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or_else(|| LabError::Parse(format!("no output pays {receiver_address}")))?;

    let change = transaction
        .outputs
        .iter()
        .find(|output| output.vout != payment.vout)
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let input_total: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();
    let output_total: f64 = transaction.outputs.iter().map(|output| output.value).sum();
    let fee = ((input_total - output_total) * 100_000_000.0).round() / 100_000_000.0;

    if fee < 0.0 {
        return Err(LabError::Parse(format!(
            "negative fee: inputs {input_total} < outputs {output_total}"
        )));
    }

    Ok(fee)
}
