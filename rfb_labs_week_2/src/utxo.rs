use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

/// Selects UTXOs to cover `target`, in the order they appear in the slice.
///
/// Takes a borrowed slice and returns borrowed UTXOs, so the caller keeps
/// ownership of its wallet and nothing is copied. The returned references cannot
/// outlive `available_utxos`, which is what stops a selection from referring to
/// coins that have since been spent or dropped.
pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    let mut selected = Vec::new();
    let mut accumulated = 0u64;

    for utxo in available_utxos {
        if accumulated >= target {
            break;
        }

        accumulated += utxo.value;
        selected.push(utxo);
    }

    if accumulated < target {
        return Err(TransactionError::InsufficientFunds {
            available: available_utxos.iter().map(|utxo| utxo.value).sum(),
            required: target,
        });
    }

    Ok(selected)
}
