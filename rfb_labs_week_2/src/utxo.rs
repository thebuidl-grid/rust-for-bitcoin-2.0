use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    
    // borrowed UTXOs and InsufficientFunds when their total is too small.

    let mut selected = Vec::new();
    let mut total = 0u64;

    for utxo in available_utxos {
        if total >= target {
            break;
        }
        selected.push(utxo);
        total += utxo.value;
    }

    if total < target {
        let available = available_utxos.iter().map(|u| u.value).sum();
        return Err(TransactionError::InsufficientFunds {
            available,
            required: target,
        });
    }

    Ok(selected)
}
