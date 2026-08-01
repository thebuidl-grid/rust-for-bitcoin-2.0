//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::RpcClient;
use crate::LabResult;
use serde::Deserialize;

/// Helper struct to bridge RPC field names (e.g., `blockhash`) to `WalletTransactionStatus`.
#[derive(Deserialize)]
struct RpcTxStatus {
    txid: String,
    confirmations: i64,
    #[serde(default)]
    fee: Option<f64>,
    #[serde(default)]
    blockhash: Option<String>,
    #[serde(default)]
    amount: Option<f64>,
}

/// Local helper to fetch wallet balances without importing Lab 02 module directly.
fn get_wallet_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let json_str = client.call(Some(wallet_name), "getbalances", &[])?;
    let val: serde_json::Value = serde_json::from_str(&json_str)?;
    let mine = &val["mine"];

    let trusted = mine["trusted"].as_f64().unwrap_or(0.0);
    let untrusted_pending = mine["untrusted_pending"].as_f64().unwrap_or(0.0);
    let immature = mine["immature"].as_f64().unwrap_or(0.0);

    Ok(WalletBalances {
        trusted,
        untrusted_pending,
        immature,
    })
}

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let raw_response = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_string(), amount_btc.to_string()],
    )?;

    let txid = serde_json::from_str::<String>(&raw_response)
        .unwrap_or_else(|_| raw_response.trim().trim_matches('"').to_string());

    Ok(txid)
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw_response = client.call(None, "getrawmempool", &[])?;
    let mempool: Vec<String> = serde_json::from_str(&raw_response)?;
    Ok(mempool)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw_response = client.call(
        Some(wallet_name),
        "gettransaction",
        &[txid.to_string()],
    )?;
    
    let raw: RpcTxStatus = serde_json::from_str(&raw_response)?;
    
    Ok(WalletTransactionStatus {
        txid: raw.txid,
        confirmations: raw.confirmations,
        fee: raw.fee,
        block_hash: raw.blockhash,
        amount: raw.amount.unwrap_or(0.0),
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
    let receiver_balance = get_wallet_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}