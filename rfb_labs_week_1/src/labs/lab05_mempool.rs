//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletTransactionStatus};
use crate::rpc::RpcClient;
use crate::LabResult;
use crate::{labs::lab03_maturity::get_balances, LabError};

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_string(), amount_btc.to_string()],
    )
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let response = client.call(None, "getrawmempool", &[])?;
    let transaction_ids = serde_json::from_str::<Vec<String>>(&response)?;

    Ok(transaction_ids)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let response = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let json: serde_json::Value = serde_json::from_str(&response)?;

    let txid = json
        .get("txid")
        .and_then(serde_json::Value::as_str)
        .ok_or(LabError::MissingField("txid"))?
        .to_string();
    let amount = json
        .get("amount")
        .and_then(serde_json::Value::as_f64)
        .ok_or(LabError::MissingField("amount"))?;
    let fee = json
        .get("fee")
        .map(|value| {
            value
                .as_f64()
                .ok_or_else(|| LabError::Parse("invalid `fee` field".to_string()))
        })
        .transpose()?;
    let confirmations = json
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let block_hash = json
        .get("blockhash")
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| LabError::Parse("invalid `blockhash` field".to_string()))
        })
        .transpose()?;

    Ok(WalletTransactionStatus {
        txid,
        confirmations,
        amount,
        fee,
        block_hash,
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
        mempool_contains_tx: mempool.contains(&txid),
        txid,
        sender_status,
        receiver_balance,
    })
}
