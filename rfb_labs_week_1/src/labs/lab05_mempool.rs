//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::RpcClient;
use crate::LabResult;

/// Send bitcoin from one wallet and return the TXID.
use crate::rpc::parse_cli_value;
use crate::LabError;
use crate::labs::lab03_maturity::get_balances;
use serde::Deserialize;

#[derive(Deserialize)]
struct RawTransactionStatus {
    txid: String,
    confirmations: i64,
    amount: f64,
    fee: Option<f64>,
    blockhash: Option<String>,
}

impl From<RawTransactionStatus> for WalletTransactionStatus {
    fn from(raw: RawTransactionStatus) -> Self {
        WalletTransactionStatus {
            txid: raw.txid,
            confirmations: raw.confirmations,
            amount: raw.amount,
            fee: raw.fee,
            block_hash: raw.blockhash,
        }
    }
}

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
        &[destination.to_owned(), amount_btc.to_string()],
    )?;
    let json = parse_cli_value(&raw)?;
    json.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("Expected txid to be a string".to_owned()))
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let json = parse_cli_value(&raw)?;
    let txids: Vec<String> = serde_json::from_value(json)?;
    Ok(txids)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let json = parse_cli_value(&raw)?;
    let raw_status: RawTransactionStatus = serde_json::from_value(json)?;
    Ok(WalletTransactionStatus::from(raw_status))
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
    let receiver_balance = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}
