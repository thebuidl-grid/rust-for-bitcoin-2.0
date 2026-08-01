//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::labs::lab03_maturity::get_balances;
use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};
use serde_json::Value;

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
        &[destination.to_string(), format!("{amount_btc}")],
    )?;
    Ok(raw.trim().trim_matches('"').to_string())
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    value
        .as_array()
        .ok_or(LabError::MissingField("getrawmempool"))?
        .iter()
        .map(|txid| txid.as_str().map(ToOwned::to_owned))
        .collect::<Option<Vec<String>>>()
        .ok_or(LabError::MissingField("getrawmempool"))
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    Ok(WalletTransactionStatus {
        txid: value["txid"]
            .as_str()
            .ok_or(LabError::MissingField("txid"))?
            .to_string(),
        confirmations: value["confirmations"]
            .as_i64()
            .ok_or(LabError::MissingField("confirmations"))?,
        amount: value["amount"]
            .as_f64()
            .ok_or(LabError::MissingField("amount"))?,
        fee: value["fee"].as_f64(),
        block_hash: value["blockhash"].as_str().map(ToOwned::to_owned),
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
    let mempool_contains_tx = mempool.contains(&txid);
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;
    let receiver_balance: WalletBalances = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}
