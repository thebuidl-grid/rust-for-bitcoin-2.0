use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    let mut selected = Vec::new();
    let mut total = 0;

    for utxo in available_utxos {
        if total >= target {
            break;
        }

        total += utxo.value;
        selected.push(utxo);
    }

    if total >= target {
        Ok(selected)
    } else {
        Err(TransactionError::InsufficientFunds {
            available: total,
            required: target,
        })
    }
}
