use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    let mut selected = Vec::new();
    let mut current_sum = 0;
    for utxo in available_utxos {
        if current_sum >= target {
            break;
        }
        selected.push(utxo);
        current_sum += utxo.value;
    }
    if current_sum < target {
        return Err(TransactionError::InsufficientFunds {
            available: current_sum,
            required: target,
        });
    }
    Ok(selected)
}
