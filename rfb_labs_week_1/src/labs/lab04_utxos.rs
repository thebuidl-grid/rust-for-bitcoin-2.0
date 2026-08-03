//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;
    let entries = value
        .as_array()
        .ok_or_else(|| LabError::Parse("listunspent did not return an array".to_owned()))?;

    entries.iter().map(decode_utxo).collect()
}

/// Decode one `listunspent` entry.
///
/// Bitcoin Core spells the locking script `scriptPubKey`, so the fields are read by
/// name rather than derived from [`Utxo`].
fn decode_utxo(value: &Value) -> LabResult<Utxo> {
    Ok(Utxo {
        txid: required_string(value, "txid")?,
        vout: u32::try_from(required_u64(value, "vout")?)
            .map_err(|error| LabError::Parse(error.to_string()))?,
        address: value
            .get("address")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        script_pub_key: required_string(value, "scriptPubKey")?,
        amount: required_f64(value, "amount")?,
        confirmations: required_u64(value, "confirmations")?,
        spendable: value
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
        // Ties break on the outpoint so the choice never depends on response order.
        .max_by(|left, right| {
            left.confirmations
                .cmp(&right.confirmations)
                .then_with(|| left.txid.cmp(&right.txid))
                .then_with(|| left.vout.cmp(&right.vout))
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
