//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde_json::Value;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    // TODO: call listunspent in wallet context and decode every returned UTXO.
    // todo!("Lab 04: list unspent outputs")
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    let entries = value.as_array().ok_or(LabError::MissingField("listunspent"))?;

    entries
        .iter()
        .map(|entry| {
            let txid = entry["txid"]
                .as_str()
                .ok_or(LabError::MissingField("txid"))?
                .to_string();
            let vout = entry["vout"]
                .as_u64()
                .ok_or(LabError::MissingField("vout"))? as u32;
            let address = entry["address"].as_str().map(|s| s.to_string());
            let script_pub_key = entry["scriptPubKey"]
                .as_str()
                .ok_or(LabError::MissingField("scriptPubKey"))?
                .to_string();
            let amount = entry["amount"]
                .as_f64()
                .ok_or(LabError::MissingField("amount"))?;
            let confirmations = entry["confirmations"]
                .as_u64()
                .ok_or(LabError::MissingField("confirmations"))?;
            let spendable = entry["spendable"]
                .as_bool()
                .ok_or(LabError::MissingField("spendable"))?;

            Ok(Utxo {
                txid,
                vout,
                address,
                script_pub_key,
                amount,
                confirmations,
                spendable,
            })
        })
        .collect()
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    // TODO: filter by spendable and select deterministically.
    // todo!("Lab 04: select a spendable UTXO")
    utxos
        .iter()
        .filter(|u| u.spendable)
        .max_by_key(|u| u.confirmations)
        .cloned()
}

/// Convert a UTXO into its unique `txid:vout` coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    // TODO: return the matching outpoint.
    // todo!("Lab 04: construct an outpoint")
    OutPoint {
        txid: utxo.txid.clone(),
        vout: utxo.vout,
    }
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    // TODO: ignore non-spendable entries and sum BTC amounts.
    // todo!("Lab 04: calculate spendable wallet balance")
    utxos
        .iter()
        .filter(|u| u.spendable)
        .map(|u| u.amount)
        .sum()
}
