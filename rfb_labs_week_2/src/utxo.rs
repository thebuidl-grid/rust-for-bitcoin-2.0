use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    let available_total: u64 = available_utxos.iter().map(|utxo| utxo.value).sum();

    if available_total < target {
        return Err(TransactionError::InsufficientFunds {
            available: available_total,
            required: target,
        });
    }

    let mut selected = Vec::new();
    let mut accumulated = 0;

    for utxo in available_utxos {
        selected.push(utxo);
        accumulated += utxo.value;

        if accumulated >= target {
            break;
        }
    }

    Ok(selected)
}
