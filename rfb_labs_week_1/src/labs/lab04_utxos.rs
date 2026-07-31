//! Lab 04 — inspect UTXOs and outpoints.

use crate::labs::lab_helper::{required_bool, required_u32};
use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    // TODO: call listunspent in wallet context and decode every returned UTXO.
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;
    let arr = value.as_array().ok_or(LabError::Parse(
        "Expected JSON array from listunspent".to_string(),
    ))?;

    let utxos = arr
        .iter()
        .map(|u| {
            Ok(Utxo {
                txid: required_string(u, "txid")?,
                vout: required_u32(u, "vout")?,
                address: u
                    .get("address")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                script_pub_key: required_string(u, "scriptPubKey")?,
                amount: required_f64(u, "amount")?,
                confirmations: required_u64(u, "confirmations")?,
                spendable: required_bool(u, "spendable")?,
            })
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(utxos)
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|u| u.spendable)
        .max_by(|a, b| {
            let c = a.confirmations.cmp(&b.confirmations);
            if c != std::cmp::Ordering::Equal {
                return c;
            }

            let tx = a.txid.cmp(&b.txid);
            if tx != std::cmp::Ordering::Equal {
                return tx;
            }

            a.vout.cmp(&b.vout)
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
