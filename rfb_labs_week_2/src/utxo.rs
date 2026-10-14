use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    // TODO(Part 9): select in slice order until the target is reached. Return
    // borrowed UTXOs and InsufficientFunds when their total is too small.

    let mut accumulated = 0u64;
    let mut selected = Vec::new();

    // Iterate through available UTXOs in slice order
    for utxo in available_utxos {
        selected.push(utxo);
        accumulated = accumulated.saturating_add(utxo.value);

        // Stop as soon as target is met or exceeded
        if accumulated >= target {
            return Ok(selected);
        }
    }

    // Return InsufficientFunds with exact satoshi context if total available < target
    Err(TransactionError::InsufficientFunds {
        available: accumulated,
        required: target,
    })
}
