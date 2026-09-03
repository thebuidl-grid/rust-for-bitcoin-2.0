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
    let response = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_owned(), amount_btc.to_string()],
    )?;
    parse_cli_value(&response)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("txid"))
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let response = client.call(None, "getrawmempool", &[])?;
    serde_json::from_value(parse_cli_value(&response)?).map_err(Into::into)
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
        txid: required_string(&value, "txid")?,
        confirmations: value
            .get("confirmations")
            .and_then(serde_json::Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        amount: required_f64(&value, "amount")?,
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
    let mempool_contains_tx = get_raw_mempool(client)?.iter().any(|id| id == &txid);
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;
    let response = client.call(Some(receiver_wallet), "getbalances", &[])?;
    let balances = parse_cli_value(&response)?;
    let mine = balances.get("mine").ok_or(LabError::MissingField("mine"))?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance: WalletBalances {
            trusted: required_f64(mine, "trusted")?,
            untrusted_pending: required_f64(mine, "untrusted_pending")?,
            immature: required_f64(mine, "immature")?,
        },
    })
}
