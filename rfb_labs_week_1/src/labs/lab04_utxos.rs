//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;

    value
        .as_array()
        .ok_or_else(|| LabError::Parse("listunspent did not return an array".to_owned()))?
        .iter()
        .map(decode_utxo)
        .collect()
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        // On equal depth fall back to the outpoint so repeated runs pick the same coin.
        .max_by(|left, right| {
            left.confirmations
                .cmp(&right.confirmations)
                .then_with(|| right.txid.cmp(&left.txid))
                .then_with(|| right.vout.cmp(&left.vout))
        })
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

/// Decode a single `listunspent` entry.
fn decode_utxo(entry: &Value) -> LabResult<Utxo> {
    Ok(Utxo {
        txid: required_string(entry, "txid")?,
        vout: required_u64(entry, "vout")? as u32,
        // Outputs paying a bare script rather than an address have no `address` field.
        address: entry
            .get("address")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        script_pub_key: required_string(entry, "scriptPubKey")?,
        amount: required_f64(entry, "amount")?,
        confirmations: required_u64(entry, "confirmations")?,
        spendable: entry
            .get("spendable")
            .and_then(Value::as_bool)
            .ok_or(LabError::MissingField("spendable"))?,
    })
}
