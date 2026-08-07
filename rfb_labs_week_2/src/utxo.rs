use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    // TODO(Part 9): select in slice order until the target is reached. Return
    // borrowed UTXOs and InsufficientFunds when their total is too small.
    let mut selected = Vec::new();
    let mut accumulated = 0;

    for utxo in available_utxos {
        selected.push(utxo);
        accumulated += utxo.value;

        if accumulated >= target {
            return Ok(selected);
        }
    }

    let total_available = available_utxos.iter().map(|utxo| utxo.value).sum();

    Err(TransactionError::InsufficientFunds {
        available: total_available,
        required: target,
    })
}
