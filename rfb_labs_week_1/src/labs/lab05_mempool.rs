//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::LabResult;

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
    let value = parse_cli_value(&response)?;

    Ok(value
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let response = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value = parse_cli_value(&response)?;

    Ok(WalletTransactionStatus {
        txid: required_string(&value, "txid")?,
        confirmations: required_u64(&value, "confirmations")? as i64,
        amount: required_f64(&value, "amount")?,
        fee: value["fee"].as_f64(),
        block_hash: value["blockhash"].as_str().map(String::from),
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

    let response = client.call(Some(receiver_wallet), "getbalances", &[])?;
    let value = parse_cli_value(&response)?;
    let mine = &value["mine"];

    let receiver_balance = WalletBalances {
        trusted: required_f64(mine, "trusted")?,
        untrusted_pending: required_f64(mine, "untrusted_pending")?,
        immature: required_f64(mine, "immature")?,
    };

    Ok(MempoolObservation {
        txid: txid.clone(),
        mempool_contains_tx: mempool.contains(&txid),
        sender_status,
        receiver_balance,
    })
}