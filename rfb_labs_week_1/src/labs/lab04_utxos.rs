//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    // TODO: call listunspent in wallet context and decode every returned UTXO.
    let call = client.call(Some(wallet_name), "listunspent", &[])?;
    let response = parse_cli_value(&call)?;

    let utxos_array = response
        .as_array()
        .ok_or_else(|| LabError::Parse("expected array".to_string()))?;

    utxos_array
        .iter()
        .map(|v| {
            // Manually extract fields to handle scriptPubKey -> script_pub_key
            let obj = v.as_object()
                .ok_or_else(|| LabError::Parse("expected object".to_string()))?;
            
            Ok(Utxo {
                txid: obj.get("txid")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| LabError::Parse("missing txid".to_string()))?
                    .to_string(),
                vout: obj.get("vout")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| LabError::Parse("missing vout".to_string()))? as u32,
                address: obj.get("address").and_then(|v| v.as_str()).map(|s| s.to_string()),
                script_pub_key: obj.get("scriptPubKey")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| LabError::Parse("missing scriptPubKey".to_string()))?
                    .to_string(),
                amount: obj.get("amount")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| LabError::Parse("missing amount".to_string()))?,
                confirmations: obj.get("confirmations")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| LabError::Parse("missing confirmations".to_string()))?,
                spendable: obj.get("spendable")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| LabError::Parse("missing spendable".to_string()))?,
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
    // TODO: return the matching outpoint
    OutPoint {
        txid: utxo.txid.clone(),
        vout: utxo.vout,
    }
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    // TODO: ignore non-spendable entries and sum BTC amounts.ut
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
