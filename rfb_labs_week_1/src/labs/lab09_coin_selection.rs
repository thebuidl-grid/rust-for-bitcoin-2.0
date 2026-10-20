//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
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
            &[alice_address.to_owned(), "0.4".to_owned()],
        )?;
        let value = parse_cli_value(&raw)?;
        let txid = value
            .as_str()
            .ok_or_else(|| LabError::Parse("Expected string TXID".to_owned()))?;
        txids.push(txid.to_owned());
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
    let arr = value
        .as_array()
        .ok_or_else(|| LabError::Parse("Expected array".to_owned()))?;
    let mut utxos = Vec::new();
    for u in arr {
        let address_val = u.get("address").and_then(Value::as_str);
        let confirmations = u
            .get("confirmations")
            .and_then(Value::as_u64)
            .ok_or_else(|| LabError::MissingField("confirmations"))?;
        if address_val == Some(address) && confirmations >= 1 {
            let txid = u
                .get("txid")
                .and_then(Value::as_str)
                .ok_or_else(|| LabError::MissingField("txid"))?
                .to_owned();
            let vout = u
                .get("vout")
                .and_then(Value::as_u64)
                .ok_or_else(|| LabError::MissingField("vout"))? as u32;
            let script_pub_key = u
                .get("scriptPubKey")
                .and_then(Value::as_str)
                .ok_or_else(|| LabError::MissingField("scriptPubKey"))?
                .to_owned();
            let amount = u
                .get("amount")
                .and_then(Value::as_f64)
                .ok_or_else(|| LabError::MissingField("amount"))?;
            let spendable = u
                .get("spendable")
                .and_then(Value::as_bool)
                .ok_or_else(|| LabError::MissingField("spendable"))?;
            utxos.push(Utxo {
                txid,
                vout,
                address: address_val.map(String::from),
                script_pub_key,
                amount,
                confirmations,
                spendable,
            });
        }
    }
    Ok(utxos)
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
        &[receiver_address.to_owned(), "1".to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;
    let txid = value
        .as_str()
        .ok_or_else(|| LabError::Parse("Expected string TXID".to_owned()))?;
    Ok(txid.to_owned())
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;
    let decoded = crate::labs::lab06_decode::decode_verbose_transaction(client, &spend_txid)?;
    let payment_and_change =
        crate::labs::lab06_decode::identify_payment_and_change(&decoded, receiver_address)?;
    let fee = crate::labs::lab06_decode::calculate_fee(&decoded)?;
    let spend_input_count = decoded.inputs.len();
    let funding_outpoints = funding_utxos.iter().map(|u| u.outpoint()).collect();
    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count,
        payment_and_change,
        fee,
    })
}
