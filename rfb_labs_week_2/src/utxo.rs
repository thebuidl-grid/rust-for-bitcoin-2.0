use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    if target == 0 {
        return Ok(Vec::new());
    }

    let mut selected = Vec::new();
    let mut accumulated = 0u64;

    for utxo in available_utxos {
        selected.push(utxo);
        accumulated += utxo.value;
        if accumulated >= target {
            return Ok(selected);
        }
    }

    Err(TransactionError::InsufficientFunds {
        available: accumulated,
        required: target,
    })
}
