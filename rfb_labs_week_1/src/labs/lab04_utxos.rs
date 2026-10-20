//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::parse_cli_value;
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let response = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&response)?;

    value
        .as_array()
        .ok_or(LabError::Parse("listunspent must return an array".to_string()))?
        .iter()
        .map(|entry| {
            Ok(Utxo {
                txid: entry
                    .get("txid")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::MissingField("txid"))?,
                vout: entry
                    .get("vout")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(LabError::MissingField("vout"))? as u32,
                address: entry
                    .get("address")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
                script_pub_key: entry
                    .get("scriptPubKey")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::MissingField("scriptPubKey"))?,
                amount: entry
                    .get("amount")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or(LabError::MissingField("amount"))?,
                confirmations: entry
                    .get("confirmations")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(LabError::MissingField("confirmations"))?,
                spendable: entry
                    .get("spendable")
                    .and_then(serde_json::Value::as_bool)
                    .ok_or(LabError::MissingField("spendable"))?,
            })
        })
        .collect()
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .max_by(|a, b| {
            a.confirmations
                .cmp(&b.confirmations)
                .then_with(|| a.txid.cmp(&b.txid))
                .then_with(|| a.vout.cmp(&b.vout))
        })
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
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
