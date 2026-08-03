//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Decode one entry of a `listunspent` response.
///
/// Shared with Lab 09, which filters the same wallet output list by address.
pub fn decode_utxo(entry: &Value) -> LabResult<Utxo> {
    Ok(Utxo {
        txid: required_string(entry, "txid")?,
        vout: required_u64(entry, "vout")? as u32,
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

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let response = parse_cli_value(&raw)?;

    let entries = response
        .as_array()
        .ok_or_else(|| LabError::Parse("expected a JSON array of unspent outputs".to_owned()))?;

    entries.iter().map(decode_utxo).collect()
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        // txid and vout break confirmation ties so repeated runs select the same coin.
        .max_by_key(|utxo| (utxo.confirmations, utxo.txid.clone(), utxo.vout))
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
