//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab06_decode::{calculate_fee, decode_verbose_transaction, identify_payment_and_change};
use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    (0..3)
        .map(|_| {
            let response = client.call(
                Some(miner_wallet),
                "sendtoaddress",
                &[alice_address.to_owned(), "0.4".to_string()],
            )?;
            parse_cli_value(&response)?
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(LabError::Parse("transaction id must be a string".to_string()))
        })
        .collect()
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    let response = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&response)?;

    let utxos = value
        .as_array()
        .ok_or(LabError::Parse("listunspent must return an array".to_string()))?
        .iter()
        .map(|entry| {
            Ok(Utxo {
                txid: entry
                    .get("txid")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::MissingField("txid"))?,
                vout: entry
                    .get("vout")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(LabError::MissingField("vout"))? as u32,
                address: entry
                    .get("address")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
                script_pub_key: entry
                    .get("scriptPubKey")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::MissingField("scriptPubKey"))?,
                amount: entry
                    .get("amount")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or(LabError::MissingField("amount"))?,
                confirmations: entry
                    .get("confirmations")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(LabError::MissingField("confirmations"))?,
                spendable: entry
                    .get("spendable")
                    .and_then(serde_json::Value::as_bool)
                    .ok_or(LabError::MissingField("spendable"))?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(utxos
        .into_iter()
        .filter(|utxo| utxo.confirmations > 0 && utxo.address.as_deref() == Some(address))
        .collect())
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    let response = client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_owned(), "1".to_string()],
    )?;

    parse_cli_value(&response)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse("transaction id must be a string".to_string()))
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;
    let decoded = decode_verbose_transaction(client, &spend_txid)?;
    let payment_and_change = identify_payment_and_change(&decoded, receiver_address)?;
    let fee = calculate_fee(&decoded)?;
    let funding_outpoints = funding_utxos.iter().map(Utxo::outpoint).collect();

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count: decoded.inputs.len(),
        payment_and_change,
        fee,
    })
}
