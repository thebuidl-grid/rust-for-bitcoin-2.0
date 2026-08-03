//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::parse_cli_value;
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let response = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_owned(), amount_btc.to_string()],
    )?;

    parse_cli_value(&response)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse("transaction id must be a string".to_string()))
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let response = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&response)?;

    value
        .as_array()
        .ok_or(LabError::Parse("getrawmempool must return an array".to_string()))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(LabError::Parse("mempool txid must be a string".to_string()))
        })
        .collect()
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let response = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&response)?;

    Ok(WalletTransactionStatus {
        txid: value
            .get("txid")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(LabError::MissingField("txid"))?,
        confirmations: value
            .get("confirmations")
            .and_then(serde_json::Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        amount: value
            .get("amount")
            .and_then(serde_json::Value::as_f64)
            .ok_or(LabError::MissingField("amount"))?,
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
    let mempool_txids = get_raw_mempool(client)?;
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;

    let receiver_balances_response = client.call(Some(receiver_wallet), "getbalances", &[])?;
    let receiver_balances_value = parse_cli_value(&receiver_balances_response)?;
    let receiver_mine = receiver_balances_value
        .get("mine")
        .ok_or(LabError::MissingField("mine"))?;

    let receiver_balance = WalletBalances {
        trusted: receiver_mine
            .get("trusted")
            .and_then(serde_json::Value::as_f64)
            .ok_or(LabError::MissingField("trusted"))?,
        untrusted_pending: receiver_mine
            .get("untrusted_pending")
            .and_then(serde_json::Value::as_f64)
            .ok_or(LabError::MissingField("untrusted_pending"))?,
        immature: receiver_mine
            .get("immature")
            .and_then(serde_json::Value::as_f64)
            .ok_or(LabError::MissingField("immature"))?,
    };

    Ok(MempoolObservation {
        txid: txid.clone(),
        mempool_contains_tx: mempool_txids.iter().any(|candidate| candidate == &txid),
        sender_status,
        receiver_balance,
    })
}
