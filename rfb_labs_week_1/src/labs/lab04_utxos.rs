//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::LabError;
use crate::LabResult;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    // TODO: call listunspent in wallet context and decode every returned UTXO.
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    let entries = value
        .as_array()
        .ok_or_else(|| crate::LabError::Parse(format!("expected an array, got: {value}")))?;

    entries
        .iter()
        .map(|entry| {
            Ok(Utxo {
                txid: crate::rpc::required_string(entry, "txid")?,
                vout: crate::rpc::required_u64(entry, "vout")? as u32,
                address: entry
                    .get("address")
                    .and_then(|v| v.as_str())
                    .map(ToOwned::to_owned),
                script_pub_key: crate::rpc::required_string(entry, "scriptPubKey")?,
                amount: crate::rpc::required_f64(entry, "amount")?,
                confirmations: crate::rpc::required_u64(entry, "confirmations")?,
                spendable: entry
                    .get("spendable")
                    .and_then(|v| v.as_bool())
                    .ok_or(LabError::MissingField("spendable"))?,
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
        .max_by_key(|utxo| utxo.confirmations)
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
