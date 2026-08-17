//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let entries = parse_cli_value(&raw)?;

    entries
        .as_array()
        .ok_or_else(|| LabError::Parse(format!("listunspent returned a non-array: {raw}")))?
        .iter()
        .map(decode_utxo)
        .collect()
}

/// Decode a single `listunspent` entry.
///
/// Bitcoin Core spells the locking script `scriptPubKey`, which does not match the
/// snake-case field on [`Utxo`], so each field is read by name.
fn decode_utxo(entry: &Value) -> LabResult<Utxo> {
    Ok(Utxo {
        txid: required_string(entry, "txid")?,
        vout: u32::try_from(required_u64(entry, "vout")?)
            .map_err(|_| LabError::Parse("vout does not fit in a u32".to_owned()))?,
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
