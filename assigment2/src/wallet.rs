use crate::error::TransactionError;
use crate::transaction::{Address, OutPoint, Sats, Transaction, TxInput, TxOutput, Validate};
use crate::utxo::{CoinSelectionStrategy, UtxoSet};

/// Owns a set of spendable UTXOs and builds transactions from them,
/// returning any leftover value to its own change address.
pub struct Wallet {
    pub change_address: Address,
    utxos: UtxoSet,
    next_tx_id: u64,
}

impl Wallet {
    pub fn new(change_address: Address) -> Self {
        Wallet {
            change_address,
            utxos: UtxoSet::new(),
            next_tx_id: 0,
        }
    }

    pub fn balance(&self) -> Sats {
        self.utxos.balance()
    }

    /// Adds an output the wallet can spend, e.g. one received from a peer.
    pub fn fund(&mut self, outpoint: OutPoint, output: TxOutput) {
        self.utxos.insert(outpoint, output);
    }

    /// Builds and validates a transaction paying `outputs`, selecting inputs
    /// from this wallet's UTXO set and covering `fee` on top of the payment.
    /// Spent UTXOs are removed and any change is credited back to the wallet.
    pub fn create_transaction(
        &mut self,
        outputs: Vec<TxOutput>,
        fee: Sats,
        strategy: CoinSelectionStrategy,
    ) -> Result<Transaction, TransactionError> {
        let payment_total = outputs
            .iter()
            .try_fold(0u64, |acc, output| acc.checked_add(output.value))
            .ok_or(TransactionError::AmountOverflow)?;
        let target = payment_total
            .checked_add(fee)
            .ok_or(TransactionError::AmountOverflow)?;

        let selection = self.utxos.select(target, strategy)?;
        for utxo in &selection.utxos {
            self.utxos.remove(&utxo.outpoint);
        }

        self.next_tx_id += 1;
        let id = format!("tx-{}", self.next_tx_id);
        let mut tx = Transaction::new(id.clone(), 0);

        for utxo in &selection.utxos {
            tx.add_input(TxInput::Regular {
                previous_output: utxo.outpoint.clone(),
            });
        }
        for output in outputs {
            tx.add_output(output);
        }
        if selection.change > 0 {
            tx.add_output(TxOutput {
                value: selection.change,
                address: self.change_address.clone(),
            });
        }
        tx.validate()?;

        if selection.change > 0 {
            let change_index = (tx.outputs.len() - 1) as u32;
            self.utxos.insert(
                OutPoint::new(id, change_index),
                tx.outputs.last().unwrap().clone(),
            );
        }

        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fund_wallet(wallet: &mut Wallet, values: &[Sats]) {
        for (i, value) in values.iter().enumerate() {
            wallet.fund(
                OutPoint::new(format!("fund{i}"), 0),
                TxOutput {
                    value: *value,
                    address: Address::from("mine"),
                },
            );
        }
    }

    #[test]
    fn create_transaction_spends_utxos_and_returns_change() {
        let mut wallet = Wallet::new(Address::from("change"));
        fund_wallet(&mut wallet, &[1_000]);

        let tx = wallet
            .create_transaction(
                vec![TxOutput {
                    value: 300,
                    address: Address::from("merchant"),
                }],
                10,
                CoinSelectionStrategy::LargestFirst,
            )
            .unwrap();

        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 2);
        assert_eq!(tx.total_output_value().unwrap(), 990);
        assert_eq!(wallet.balance(), 690);
    }

    #[test]
    fn create_transaction_omits_change_output_on_exact_spend() {
        let mut wallet = Wallet::new(Address::from("change"));
        fund_wallet(&mut wallet, &[310]);

        let tx = wallet
            .create_transaction(
                vec![TxOutput {
                    value: 300,
                    address: Address::from("merchant"),
                }],
                10,
                CoinSelectionStrategy::LargestFirst,
            )
            .unwrap();

        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(wallet.balance(), 0);
    }

    #[test]
    fn create_transaction_fails_on_insufficient_funds_and_leaves_wallet_untouched() {
        let mut wallet = Wallet::new(Address::from("change"));
        fund_wallet(&mut wallet, &[50]);

        let err = wallet
            .create_transaction(
                vec![TxOutput {
                    value: 300,
                    address: Address::from("merchant"),
                }],
                0,
                CoinSelectionStrategy::LargestFirst,
            )
            .unwrap_err();

        assert_eq!(
            err,
            TransactionError::InsufficientFunds {
                required: 300,
                available: 50
            }
        );
        assert_eq!(wallet.balance(), 50);
    }

    #[test]
    fn change_is_spendable_in_a_later_transaction() {
        let mut wallet = Wallet::new(Address::from("change"));
        fund_wallet(&mut wallet, &[1_000]);

        wallet
            .create_transaction(
                vec![TxOutput {
                    value: 100,
                    address: Address::from("a"),
                }],
                0,
                CoinSelectionStrategy::LargestFirst,
            )
            .unwrap();
        assert_eq!(wallet.balance(), 900);

        let tx2 = wallet
            .create_transaction(
                vec![TxOutput {
                    value: 800,
                    address: Address::from("b"),
                }],
                0,
                CoinSelectionStrategy::LargestFirst,
            )
            .unwrap();
        assert_eq!(tx2.total_output_value().unwrap(), 900);
        assert_eq!(wallet.balance(), 100);
    }
}
