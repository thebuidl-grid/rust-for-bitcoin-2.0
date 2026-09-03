//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let response = client.call(Some(wallet_name), "listunspent", &[])?;
    let entries = parse_cli_value(&response)?;
    let entries = entries
        .as_array()
        .ok_or(LabError::MissingField("listunspent array"))?;

    entries
        .iter()
        .map(|entry| {
            let vout = required_u64(entry, "vout")?
                .try_into()
                .map_err(|_| LabError::Parse("vout does not fit in u32".to_owned()))?;
            Ok(Utxo {
                txid: required_string(entry, "txid")?,
                vout,
                address: entry
                    .get("address")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
                script_pub_key: required_string(entry, "scriptPubKey")?,
                amount: required_f64(entry, "amount")?,
                confirmations: required_u64(entry, "confirmations")?,
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
