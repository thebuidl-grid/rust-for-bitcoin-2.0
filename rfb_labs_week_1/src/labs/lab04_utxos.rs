//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
use serde_json::Value;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;
    let arr = value
        .as_array()
        .ok_or_else(|| LabError::Parse("Expected array of UTXOs".to_owned()))?;
    let mut utxos = Vec::new();
    for u in arr {
        let txid = u
            .get("txid")
            .and_then(Value::as_str)
            .ok_or_else(|| LabError::MissingField("txid"))?
            .to_owned();
        let vout = u
            .get("vout")
            .and_then(Value::as_u64)
            .ok_or_else(|| LabError::MissingField("vout"))? as u32;
        let address = u.get("address").and_then(Value::as_str).map(String::from);
        let script_pub_key = u
            .get("scriptPubKey")
            .and_then(Value::as_str)
            .ok_or_else(|| LabError::MissingField("scriptPubKey"))?
            .to_owned();
        let amount = u
            .get("amount")
            .and_then(Value::as_f64)
            .ok_or_else(|| LabError::MissingField("amount"))?;
        let confirmations = u
            .get("confirmations")
            .and_then(Value::as_u64)
            .ok_or_else(|| LabError::MissingField("confirmations"))?;
        let spendable = u
            .get("spendable")
            .and_then(Value::as_bool)
            .ok_or_else(|| LabError::MissingField("spendable"))?;
        utxos.push(Utxo {
            txid,
            vout,
            address,
            script_pub_key,
            amount,
            confirmations,
            spendable,
        });
    }
    Ok(utxos)
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|u| u.spendable)
        .max_by_key(|u| u.confirmations)
        .cloned()
}

/// Convert a UTXO into its unique `txid:vout` coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    OutPoint {
        txid: utxo.txid.clone(),
        vout: utxo.vout,
    }
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos.iter().filter(|u| u.spendable).map(|u| u.amount).sum()
}
