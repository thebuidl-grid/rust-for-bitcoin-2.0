//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{
    DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange, WalletBalances,
};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    // TODO: call getrawtransaction with verbosity 2 and decode:
    // - txid and vsize
    // - each vin's txid, vout, and prevout.value
    // - each vout's n, value, scriptPubKey.hex, and optional address
    // todo!("Lab 06: decode a verbose raw transaction")
    let result = client.call(
        None,
        "getrawtransaction",
        &[txid.to_string(), 2.to_string()],
    );

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let vsize = parsed["vsize"].clone();
            let inputs = parsed["vin"]
                .as_array()
                .unwrap()
                .iter()
                .map(|vin| DecodedInput {
                    previous_output: OutPoint {
                        txid: vin["txid"].as_str().unwrap_or_default().to_owned(),
                        vout: vin["vout"].as_u64().unwrap_or_default() as u32,
                    },
                    previous_value: vin["prevout"]["value"].as_f64().unwrap_or_default(),
                })
                .collect();
            let outputs = parsed["vout"]
                .as_array()
                .unwrap()
                .iter()
                .map(|vout| DecodedOutput {
                    vout: vout["n"].as_u64().unwrap_or_default() as u32,
                    value: vout["value"].as_f64().unwrap_or_default(),
                    address: vout["scriptPubKey"]["address"]
                        .as_str()
                        .map(ToOwned::to_owned),
                    script_pub_key_hex: vout["scriptPubKey"]["hex"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                })
                .collect();

            Ok(DecodedTransaction {
                txid: parsed["txid"].as_str().unwrap_or_default().to_owned(),
                inputs,
                outputs,
                vsize: parsed["vsize"].as_u64().unwrap_or_default(),
            })
        }
        Err(err) => Err(err),
    }
}

/// Return every previous output consumed by the transaction.
pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
    // TODO: map decoded inputs to their outpoints.
    // todo!("Lab 06: list consumed outpoints")
    transaction
        .inputs
        .iter()
        .map(|input| input.previous_output.clone())
        .collect::<Vec<OutPoint>>()
}

/// Identify the receiver payment and optional change output.
pub fn identify_payment_and_change(
    transaction: &DecodedTransaction,
    receiver_address: &str,
) -> LabResult<PaymentAndChange> {
    // TODO: match the receiver address; treat the remaining non-OP_RETURN output as change.
    // todo!("Lab 06: identify payment and change")
    let mut payment = None;
    let mut change = None;

    for output in &transaction.outputs {
        if output.address.as_deref() == Some(receiver_address) {
            payment = Some(output.clone());
        } else if output.script_pub_key_hex != "6a" {
            change = Some(output.clone());
        }
    }

    Ok(PaymentAndChange {
        payment: payment.ok_or(LabError::MissingField("receiver output"))?,
        change,
    })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    // TODO: reject impossible negative fees and return the BTC fee.
    // todo!("Lab 06: calculate the miner fee")
    let input_sum: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();

    let output_sum: f64 = transaction.outputs.iter().map(|output| output.value).sum();

    let fee = (input_sum - output_sum) * 1e8;
    let fee = fee.round() / 1e8;

    if fee < 0.0 {
        return Err(LabError::Parse(format!("impossible negative fee: {fee}")));
    }

    Ok(fee)
}
