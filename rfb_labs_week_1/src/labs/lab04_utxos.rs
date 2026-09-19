//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let json_str = client.call(Some(wallet_name), "listunspent", &[])?;
    let utxos: Vec<Utxo> = serde_json::from_str(&json_str)?;
    Ok(utxos)
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|u| u.spendable)
        .max_by_key(|u| u.confirmations)
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
        .filter(|u| u.spendable)
        .map(|u| u.amount)
        .sum()
}