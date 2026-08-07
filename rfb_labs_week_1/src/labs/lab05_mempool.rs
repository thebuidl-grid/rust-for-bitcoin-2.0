//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde_json::Value;

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    // TODO: call sendtoaddress in the sender's wallet context.
    // todo!("Lab 05: send bitcoin")
    let raw = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_string(), amount_btc.to_string()],
    )?;

    Ok(raw.trim().trim_matches('"').to_string())
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // TODO: call getrawmempool and decode its array.
    // todo!("Lab 05: inspect the local mempool")
    let raw = client.call(None, "getrawmempool", &[])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    value
        .as_array()
        .ok_or(LabError::MissingField("getrawmempool"))?
        .iter()
        .map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Option<Vec<String>>>()
        .ok_or(LabError::MissingField("getrawmempool"))
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    // TODO: call gettransaction and decode txid, amount, fee, confirmations, and blockhash.
    // todo!("Lab 05: inspect wallet transaction status")
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    let txid = value["txid"]
        .as_str()
        .ok_or(LabError::MissingField("txid"))?
        .to_string();
    let confirmations = value["confirmations"]
        .as_i64()
        .ok_or(LabError::MissingField("confirmations"))?;
    let amount = value["amount"]
        .as_f64()
        .ok_or(LabError::MissingField("amount"))?;
    let fee = value["fee"].as_f64();
    let block_hash = value["blockhash"].as_str().map(|s| s.to_string());

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
    // TODO: send, inspect getrawmempool, inspect sender status, and read receiver balances.
    // todo!("Lab 05: prove a payment is broadcast but unconfirmed")
     let txid = send_btc(client, sender_wallet, receiver_address, amount_btc)?;

    let mempool = get_raw_mempool(client)?;
    let mempool_contains_tx = mempool.iter().any(|id| id == &txid);

    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;

    let raw_balances = client.call(Some(receiver_wallet), "getbalances", &[])?;
    let balances_value: Value =
        serde_json::from_str(&raw_balances).map_err(|e| LabError::Parse(e.to_string()))?;
    let mine = &balances_value["mine"];

    let receiver_balance = WalletBalances {
        trusted: mine["trusted"]
            .as_f64()
            .ok_or(LabError::MissingField("mine.trusted"))?,
        untrusted_pending: mine["untrusted_pending"]
            .as_f64()
            .ok_or(LabError::MissingField("mine.untrusted_pending"))?,
        immature: mine["immature"]
            .as_f64()
            .ok_or(LabError::MissingField("mine.immature"))?,
    };

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}
