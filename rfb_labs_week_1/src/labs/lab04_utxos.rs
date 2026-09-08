//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64};
use crate::LabError;
use serde_json::Value;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let call = client.call(Some(wallet_name), "listunspent", &[])?;
    let value: Value = serde_json::from_str(&call).map_err(|e| LabError::Parse(e.to_string()))?;

    let entries = value
        .as_array()
        .ok_or(LabError::MissingField("listunspent"))?;

    entries
        .iter()
        .map(|entry| {
            let txid = required_string(entry, "txid")?;

            let vout = required_u64(entry, "vout")?;

            let address = required_string(entry, "address")?;

            let script_pub_key = required_string(entry, "scriptPubKey")?;

            let amount = required_f64(entry, "amount")?;

            let confirmations = required_u64(entry, "confirmations")?;

            let spendable = entry["spendable"]
                .as_bool()
                .ok_or(LabError::MissingField("spendable"))?;

            Ok(Utxo {
                txid,
                vout: vout.try_into().unwrap(),
                address: Some(address),
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
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .max_by_key(|utxo| utxo.confirmations)
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
