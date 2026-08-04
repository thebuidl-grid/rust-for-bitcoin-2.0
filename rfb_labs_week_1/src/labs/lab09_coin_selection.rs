//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use crate::model::{DecodedOutput, MultiUtxoAudit, OutPoint, PaymentAndChange, Utxo};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut txids = Vec::new();

    for _ in 0..3 {
        let raw = client.call(
            Some(miner_wallet),
            "sendtoaddress",
            &[alice_address.to_string(), "0.4".to_string()],
        )?;
        let value = parse_cli_value(&raw)?;

        let txid = value
            .as_str()
            .map(ToOwned::to_owned)
            .ok_or(LabError::Parse("expected txid string".to_owned()))?;
        txids.push(txid);
    }

    Ok(txids)
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;

    match value {
        Value::Array(items) => items
            .into_iter()
            .filter(|item| item.get("address").and_then(Value::as_str) == Some(address))
            .map(|item| {
                Ok(Utxo {
                    txid: item
                        .get("txid")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .ok_or(LabError::MissingField("txid"))?,
                    vout: item
                        .get("vout")
                        .and_then(Value::as_u64)
                        .map(|n| n as u32)
                        .ok_or(LabError::MissingField("vout"))?,
                    address: item
                        .get("address")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    script_pub_key: item
                        .get("scriptPubKey")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .ok_or(LabError::MissingField("scriptPubKey"))?,
                    amount: item
                        .get("amount")
                        .and_then(Value::as_f64)
                        .ok_or(LabError::MissingField("amount"))?,
                    confirmations: item
                        .get("confirmations")
                        .and_then(Value::as_u64)
                        .ok_or(LabError::MissingField("confirmations"))?,
                    spendable: item
                        .get("spendable")
                        .and_then(Value::as_bool)
                        .ok_or(LabError::MissingField("spendable"))?,
                })
            })
            .collect(),
        other => Err(LabError::Parse(format!("expected array, got {other}"))),
    }
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    let raw = client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_string(), "1".to_string()],
    )?;
    let value = parse_cli_value(&raw)?;

    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse("expected txid string".to_owned()))
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;
    let transaction = decode_verbose_transaction(client, &spend_txid)?;
    let payment_and_change = identify_payment_and_change(&transaction, receiver_address)?;
    let fee = calculate_fee(&transaction)?;
    let funding_outpoints = funding_utxos.iter().map(|utxo| utxo.outpoint()).collect();

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count: transaction.inputs.len(),
        payment_and_change,
        fee,
    })
}
