use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    let mut selected = Vec::new();
    let mut total = 0;

    for utxo in available_utxos.iter() {
        selected.push(utxo);
        total += utxo.value;
        if total >= target {
            break;
        }
    }

    if total < target {
        return Err(TransactionError::InsufficientFunds {
            available: total,
            required: target,
        });
    }

    Ok(selected)
}
