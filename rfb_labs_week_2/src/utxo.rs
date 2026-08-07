use crate::{error::TransactionError, transaction::OutPoint};

#[derive(Debug, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
}

pub fn select_utxos(available_utxos: &[Utxo], target: u64) -> Result<Vec<&Utxo>, TransactionError> {
     // TODO(Part 9): select in slice order until the target is reached. Return
    // borrowed UTXOs and InsufficientFunds when their total is too small.
   
    let _ = (available_utxos, target);
    // todo!("select UTXOs")
    // let _ = (available_utxos, target);
    
    let mut selected = Vec::new();
    let mut total = 0u64;

    for utxo in available_utxos {
        selected.push(utxo);
        total = total.saturating_add(utxo.value);

        if total >= target {
            return Ok(selected);
        }
    }

    Err(TransactionError::InsufficientFunds {
        available: total,
        required: target,
    })
}
