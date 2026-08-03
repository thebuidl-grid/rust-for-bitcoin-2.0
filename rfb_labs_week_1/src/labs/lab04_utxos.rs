//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::LabResult;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;
    value
        .as_array()
        .ok_or_else(|| crate::LabError::Parse("expected an array of UTXOs".to_owned()))?
        .iter()
        .map(utxo_from_value)
        .collect()
}

/// Bitcoin Core reports `listunspent` fields in camelCase; map them onto [`Utxo`] by hand.
fn utxo_from_value(v: &serde_json::Value) -> LabResult<Utxo> {
    Ok(Utxo {
        txid: required_string(v, "txid")?,
        vout: required_u64(v, "vout")? as u32,
        address: v.get("address").and_then(|a| a.as_str()).map(str::to_owned),
        script_pub_key: required_string(v, "scriptPubKey")?,
        amount: required_f64(v, "amount")?,
        confirmations: required_u64(v, "confirmations")?,
        spendable: v
            .get("spendable")
            .and_then(|s| s.as_bool())
            .ok_or(crate::LabError::MissingField("spendable"))?,
    })
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
    utxo.outpoint()
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos.iter().filter(|u| u.spendable).map(|u| u.amount).sum()
}
