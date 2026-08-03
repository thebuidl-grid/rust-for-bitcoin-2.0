use crate::labs::lab03_maturity::get_balances;
use crate::model::{MempoolObservation, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

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

pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;

    value
        .as_array()
        .ok_or_else(|| LabError::Parse("expected a JSON array of txids".to_owned()))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| LabError::Parse("expected a txid string".to_owned()))
        })
        .collect()
}

pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;

    Ok(WalletTransactionStatus {
        txid: required_string(&value, "txid")?,
        confirmations: value
            .get("confirmations")
            .and_then(Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        amount: required_f64(&value, "amount")?,
        fee: value.get("fee").and_then(Value::as_f64),
        block_hash: value
            .get("blockhash")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

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
