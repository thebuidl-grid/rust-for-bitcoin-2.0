//! Lab 05 — broadcast a transaction and observe the mempool.

use serde_json::Value;

use crate::labs::lab03_maturity::get_balances;
use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

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
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;
    let txids = value
        .as_array()
        .ok_or(LabError::Parse("expected JSON array".to_string()))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or(LabError::Parse("expected string".to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(txids)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value = parse_cli_value(&raw)?;

    let txid_out = required_string(&value, "txid")?;
    let confirmations = required_u64(&value, "confirmations")? as i64;
    let amount = required_f64(&value, "amount")?;

    let fee = value.get("fee").and_then(Value::as_f64);

    let block_hash = value
        .get("blockhash")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    Ok(WalletTransactionStatus {
        txid: txid_out,
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
    let mempool_txids = get_raw_mempool(client)?;
    let mempool_contains_tx = mempool_txids.iter().any(|t| t == &txid);
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;
    let receiver_balance = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}
