//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::labs::lab03_maturity::get_balances;
use crate::model::{MempoolObservation, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::LabError;
use crate::LabResult;

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let raw = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_owned(), format!("{amount_btc}")],
    )?;
    let value = parse_cli_value(&raw)?;
    match value {
        serde_json::Value::String(txid) => Ok(txid),
        _ => Err(LabError::Parse(
            "sendtoaddress did not return a string".to_owned(),
        )),
    }
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;
    let txids = value
        .as_array()
        .ok_or_else(|| LabError::Parse("getrawmempool did not return an array".to_owned()))?;

    txids
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| LabError::Parse("getrawmempool entry was not a string".to_owned()))
        })
        .collect()
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;

    Ok(WalletTransactionStatus {
        txid: required_string(&value, "txid")?,
        confirmations: value
            .get("confirmations")
            .and_then(serde_json::Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        amount: required_f64(&value, "amount")?,
        fee: value.get("fee").and_then(serde_json::Value::as_f64),
        block_hash: value
            .get("blockhash")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
    })
}

/// Send a payment without mining and capture its mempool and receiver-wallet state.
pub fn observe_unconfirmed_payment<C: RpcClient>(
    client: &C,
    sender_wallet: &str,
    receiver_wallet: &str,
    receiver_address: &str,
    amount_btc: f64,
) -> LabResult<MempoolObservation> {
    let txid = send_btc(client, sender_wallet, receiver_address, amount_btc)?;
    let mempool = get_raw_mempool(client)?;
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;
    let receiver_balance = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid: txid.clone(),
        mempool_contains_tx: mempool.iter().any(|entry| entry == &txid),
        sender_status,
        receiver_balance,
    })
}
