use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
    // TODO(Part 9): select in slice order until the target is reached. Return
    // borrowed UTXOs and InsufficientFunds when their total is too small.
    // let _ = (available_utxos, target);

    let mut selected_utxos = Vec::new();
    let mut sum = 0_u64;

    for utxo in available_utxos {
        if sum >= target {
            break;
        }

        selected_utxos.push(utxo);
        sum += utxo.value;
    }

    if sum >= target {
        Ok(selected_utxos)
    } else {
        Err(TransactionError::InsufficientFunds {
            available: sum,
            required: target,
        })
    }
}
