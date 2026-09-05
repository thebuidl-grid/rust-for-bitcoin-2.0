//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let call = client.call(Some(wallet_name), "listunspent", &[])?;
    let val = parse_cli_value(&call)?;

    let array = val
        .as_array()
        .ok_or_else(|| LabError::Parse("listunspent response is not an array".to_string()))?;

    let mut utxos = Vec::new();

    for item in array {
        let txid = required_string(item, "txid")?;
        let vout = required_u64(item, "vout")? as u32;

        let address = item
            .get("address")
            .and_then(|v| v.as_str())
            .map(String::from);

        let script_pub_key = match item.get("scriptPubKey") {
            Some(v) if v.is_string() => v.as_str().unwrap_or_default().to_string(),
            Some(v) if v.is_object() => v
                .get("hex")
                .and_then(|h| h.as_str())
                .unwrap_or_default()
                .to_string(),
            _ => String::new(),
        };

        let amount = item
            .get("amount")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| LabError::MissingField("amount"))?;

        let confirmations = required_u64(item, "confirmations")?;
        let spendable = item
            .get("spendable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        utxos.push(Utxo {
            txid,
            vout,
            address,
            script_pub_key,
            amount,
            confirmations,
            spendable,
        });
    }

    Ok(utxos)
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|u| u.spendable)
        .max_by(|a, b| {
            a.confirmations
                .cmp(&b.confirmations)
                .then_with(|| {
                    a.amount
                        .partial_cmp(&b.amount)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.txid.cmp(&b.txid))
                .then_with(|| a.vout.cmp(&b.vout))
        })
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
    utxos.iter().filter(|u| u.spendable).map(|u| u.amount).sum()
}
