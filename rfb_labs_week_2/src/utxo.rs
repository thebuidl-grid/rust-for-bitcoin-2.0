use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    let mut selected = Vec::new();
    let mut total = 0;

    if target == 0 {
        return Ok(selected);
    }

    for item in available_utxos {
        total += item.value;
        selected.push(item);

        if total >= target {
            return Ok(selected);
        }
    }

    Err(TransactionError::InsufficientFunds {
        available: total,
        required: target,
    })
}
