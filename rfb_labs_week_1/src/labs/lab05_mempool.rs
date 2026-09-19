//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};

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
    parse_cli_value(&raw)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse(
            "sendtoaddress did not return a txid".to_owned(),
        ))
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    parse_cli_value(&raw)?
        .as_array()
        .ok_or(LabError::Parse(
            "getrawmempool did not return an array".to_owned(),
        ))?
        .iter()
        .map(|txid| {
            txid.as_str()
                .map(ToOwned::to_owned)
                .ok_or(LabError::Parse("mempool txid was not a string".to_owned()))
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
    let confirmations = value
        .get("confirmations")
        .and_then(|confirmations| confirmations.as_i64())
        .ok_or(LabError::MissingField("confirmations"))?;

    Ok(WalletTransactionStatus {
        txid: required_string(&value, "txid")?,
        confirmations,
        amount: required_f64(&value, "amount")?,
        fee: value.get("fee").and_then(|fee| fee.as_f64()),
        block_hash: value
            .get("blockhash")
            .and_then(|block_hash| block_hash.as_str())
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
    let receiver_balance = crate::labs::lab03_maturity::get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        mempool_contains_tx: mempool.iter().any(|mempool_txid| mempool_txid == &txid),
        txid,
        sender_status,
        receiver_balance,
    })
}
