//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};
use serde_json::Value;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    value
        .as_array()
        .ok_or(LabError::MissingField("listunspent"))?
        .iter()
        .map(|entry| {
            Ok(Utxo {
                txid: entry["txid"]
                    .as_str()
                    .ok_or(LabError::MissingField("txid"))?
                    .to_string(),
                vout: entry["vout"]
                    .as_u64()
                    .ok_or(LabError::MissingField("vout"))? as u32,
                address: entry["address"].as_str().map(ToOwned::to_owned),
                script_pub_key: entry["scriptPubKey"]
                    .as_str()
                    .ok_or(LabError::MissingField("scriptPubKey"))?
                    .to_string(),
                amount: entry["amount"]
                    .as_f64()
                    .ok_or(LabError::MissingField("amount"))?,
                confirmations: entry["confirmations"]
                    .as_u64()
                    .ok_or(LabError::MissingField("confirmations"))?,
                spendable: entry["spendable"]
                    .as_bool()
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
        .max_by_key(|utxo| utxo.confirmations)
        .cloned()
}

/// Convert a UTXO into its unique `txid:vout` coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    utxo.outpoint()
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
