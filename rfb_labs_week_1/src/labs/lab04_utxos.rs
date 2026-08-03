//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError,LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    // TODO: call listunspent in wallet context and decode every returned UTXO.
    let raw_info = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw_info)?;

    let entries = value
        .as_array()
        .ok_or(LabError::MissingField("listunspent"))?;

    entries
        .iter()
        .map(|entry| {
            let vout = entry
                .get("vout")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as u32)
                .ok_or(LabError::MissingField("vout"))?;

            let spendable = entry
                .get("spendable")
                .and_then(serde_json::Value::as_bool)
                .ok_or(LabError::MissingField("spendable"))?;

            let address = entry
                .get("address")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned);

            Ok(Utxo {
                txid: required_string(entry, "txid")?,
                vout,
                address,
                script_pub_key: required_string(entry, "scriptPubKey")?,
                amount: required_f64(entry, "amount")?,
                confirmations: required_u64(entry, "confirmations")?,
                spendable,
            })
        })
        .collect()
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    // TODO: filter by spendable and select deterministically.
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
    // TODO: return the matching outpoint.
    utxo.outpoint()
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    // TODO: ignore non-spendable entries and sum BTC amounts.
      utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
