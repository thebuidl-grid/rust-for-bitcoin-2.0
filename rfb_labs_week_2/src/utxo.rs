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
    // todo!("select UTXOs")
    let mut selected: Vec<&Utxo> = Vec::new();
    let mut total: u64 = 0;

    for utxo in available_utxos {
        if total >= target {
            break;
        }
        selected.push(utxo);
        total += utxo.value;
    }

    if total < target {
        let available: u64 = available_utxos.iter().map(|utxo| utxo.value).sum();
        return Err(TransactionError::InsufficientFunds {
            available,
            required: target,
        });
    }

    Ok(selected)
}
