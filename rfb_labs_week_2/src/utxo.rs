use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    let mut selected = Vec::new();
    let mut total_selected = 0;

    for utxo in available_utxos {
        selected.push(utxo);
        total_selected += utxo.value;

        if total_selected >= target {
            return Ok(selected);
        }
    }

    Err(TransactionError::InsufficientFunds {
        available: total_selected,
        required: target,
    })
}
