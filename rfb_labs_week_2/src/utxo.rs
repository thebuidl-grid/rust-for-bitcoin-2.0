use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    // Greedy in slice order: keep taking UTXOs until the target is covered.
    // We return borrowed references so the caller's wallet keeps ownership.
    let mut selected = Vec::new();
    let mut running_total = 0u64;

    for utxo in available_utxos {
        selected.push(utxo);
        running_total += utxo.value;
        if running_total >= target {
            return Ok(selected);
        }
    }

    Err(TransactionError::InsufficientFunds {
        available: running_total,
        required: target,
    })
}
