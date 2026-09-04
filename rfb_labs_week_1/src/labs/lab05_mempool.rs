//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};

use super::lab03_maturity::get_balances;

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
        &[destination.to_owned(), amount_btc.to_string()],
    )
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let response = client.call(None, "getrawmempool", &[])?;
    Ok(serde_json::from_str(&response)?)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let response = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let parsed_response = parse_cli_value(&response)?;

    let confirmations = parsed_response
        .get("confirmations")
        .and_then(|value| value.as_i64())
        .ok_or(LabError::MissingField("confirmations"))?;
    let fee = parsed_response
        .get("fee")
        .map(|value| value.as_f64().ok_or(LabError::MissingField("fee")))
        .transpose()?;
    let block_hash = parsed_response
        .get("blockhash")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);

    Ok(WalletTransactionStatus {
        txid: required_string(&parsed_response, "txid")?,
        confirmations,
        amount: required_f64(&parsed_response, "amount")?,
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
