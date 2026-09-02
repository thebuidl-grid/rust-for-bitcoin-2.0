use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

/// Walks the slice in order and takes UTXOs until `target` is covered.
///
/// The returned vector borrows from `available_utxos`, so nothing is cloned and
/// the caller keeps ownership of the wallet's UTXO set.
pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    let mut selected = Vec::new();
    let mut selected_value = 0u64;

    for utxo in available_utxos {
        if selected_value >= target {
            break;
        }

        selected_value = selected_value.saturating_add(utxo.value);
        selected.push(utxo);
    }

    if selected_value < target {
        return Err(TransactionError::InsufficientFunds {
            available: selected_value,
            required: target,
        });
    }

    Ok(selected)
}
