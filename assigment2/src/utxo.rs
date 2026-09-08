use std::collections::HashMap;

use crate::error::TransactionError;
use crate::transaction::{BitcoinValue, OutPoint, Sats, TxOutput};

/// An unspent output, paired with the OutPoint that identifies it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub output: TxOutput,
}

impl BitcoinValue for Utxo {
    fn value_sats(&self) -> Sats {
        self.output.value
    }
}

/// Strategy used to pick which UTXOs cover a target amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoinSelectionStrategy {
    /// Spend the biggest UTXOs first: fewer inputs, smaller tx, but shreds big
    /// UTXOs into change and tends to leave a wallet full of dust over time.
    LargestFirst,
    /// Spend the smallest UTXOs first: consolidates dust as a side effect of
    /// spending, at the cost of larger transactions (more inputs, higher fees).
    SmallestFirst,
}

/// The result of a successful coin selection: which UTXOs were chosen, and
/// how much of their combined value is left over as change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub utxos: Vec<Utxo>,
    pub total: Sats,
    pub change: Sats,
}

/// The set of outputs a wallet is currently able to spend.
#[derive(Debug, Clone, Default)]
pub struct UtxoSet {
    utxos: HashMap<OutPoint, TxOutput>,
}

impl UtxoSet {
    pub fn new() -> Self {
        UtxoSet {
            utxos: HashMap::new(),
        }
    }

    pub fn insert(&mut self, outpoint: OutPoint, output: TxOutput) {
        self.utxos.insert(outpoint, output);
    }

    pub fn remove(&mut self, outpoint: &OutPoint) -> Option<TxOutput> {
        self.utxos.remove(outpoint)
    }

    pub fn get(&self, outpoint: &OutPoint) -> Option<&TxOutput> {
        self.utxos.get(outpoint)
    }

    pub fn len(&self) -> usize {
        self.utxos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.utxos.is_empty()
    }

    pub fn balance(&self) -> Sats {
        self.utxos.values().map(TxOutput::value_sats).sum()
    }

    /// Selects enough UTXOs to cover `target` sats, in the order given by
    /// `strategy`. Returns `InsufficientFunds` if the whole set can't cover it.
    pub fn select(
        &self,
        target: Sats,
        strategy: CoinSelectionStrategy,
    ) -> Result<Selection, TransactionError> {
        let mut candidates: Vec<Utxo> = self
            .utxos
            .iter()
            .map(|(outpoint, output)| Utxo {
                outpoint: outpoint.clone(),
                output: output.clone(),
            })
            .collect();

        match strategy {
            CoinSelectionStrategy::LargestFirst => candidates.sort_by(|a, b| {
                b.output
                    .value
                    .cmp(&a.output.value)
                    .then_with(|| a.outpoint.txid.cmp(&b.outpoint.txid))
            }),
            CoinSelectionStrategy::SmallestFirst => candidates.sort_by(|a, b| {
                a.output
                    .value
                    .cmp(&b.output.value)
                    .then_with(|| a.outpoint.txid.cmp(&b.outpoint.txid))
            }),
        }

        let mut total: Sats = 0;
        let mut chosen = Vec::new();
        for utxo in candidates {
            if total >= target {
                break;
            }
            total = total
                .checked_add(utxo.output.value)
                .ok_or(TransactionError::AmountOverflow)?;
            chosen.push(utxo);
        }

        if total < target {
            return Err(TransactionError::InsufficientFunds {
                required: target,
                available: self.balance(),
            });
        }

        Ok(Selection {
            utxos: chosen,
            total,
            change: total - target,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Address;

    fn utxo_set(values: &[Sats]) -> UtxoSet {
        let mut set = UtxoSet::new();
        for (i, value) in values.iter().enumerate() {
            set.insert(
                OutPoint::new(format!("tx{i}"), 0),
                TxOutput {
                    value: *value,
                    address: Address::from("addr"),
                },
            );
        }
        set
    }

    #[test]
    fn balance_sums_all_utxos() {
        let set = utxo_set(&[100, 200, 300]);
        assert_eq!(set.balance(), 600);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn largest_first_prefers_fewer_bigger_utxos() {
        let set = utxo_set(&[10, 50, 100, 500]);
        let selection = set
            .select(120, CoinSelectionStrategy::LargestFirst)
            .unwrap();
        assert_eq!(selection.utxos.len(), 1);
        assert_eq!(selection.total, 500);
        assert_eq!(selection.change, 380);
    }

    #[test]
    fn smallest_first_consolidates_dust() {
        let set = utxo_set(&[10, 50, 100, 500]);
        let selection = set
            .select(120, CoinSelectionStrategy::SmallestFirst)
            .unwrap();
        assert_eq!(selection.utxos.len(), 3);
        assert_eq!(selection.total, 160);
        assert_eq!(selection.change, 40);
    }

    #[test]
    fn selecting_exact_balance_leaves_no_change() {
        let set = utxo_set(&[100, 200]);
        let selection = set
            .select(300, CoinSelectionStrategy::LargestFirst)
            .unwrap();
        assert_eq!(selection.total, 300);
        assert_eq!(selection.change, 0);
    }

    #[test]
    fn selection_fails_when_funds_are_insufficient() {
        let set = utxo_set(&[10, 20]);
        let err = set
            .select(1_000, CoinSelectionStrategy::LargestFirst)
            .unwrap_err();
        assert_eq!(
            err,
            TransactionError::InsufficientFunds {
                required: 1_000,
                available: 30
            }
        );
    }

    #[test]
    fn selection_on_empty_set_fails() {
        let set = UtxoSet::new();
        let err = set
            .select(1, CoinSelectionStrategy::SmallestFirst)
            .unwrap_err();
        assert_eq!(
            err,
            TransactionError::InsufficientFunds {
                required: 1,
                available: 0
            }
        );
    }

    #[test]
    fn get_and_remove_round_trip() {
        let mut set = utxo_set(&[42]);
        let outpoint = OutPoint::new("tx0", 0);
        assert!(set.get(&outpoint).is_some());
        let removed = set.remove(&outpoint).unwrap();
        assert_eq!(removed.value, 42);
        assert!(set.get(&outpoint).is_none());
        assert!(set.is_empty());
    }
}
