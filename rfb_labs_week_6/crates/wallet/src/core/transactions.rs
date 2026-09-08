use std::{str::FromStr, time::UNIX_EPOCH};

use bdk_wallet::{SignOptions, psbt::PsbtUtils};
use bitcoin::{Address, Amount, FeeRate};

use crate::{
    core::WalletService,
    error::{WalletError, WalletResult},
    node::NodeBackend,
    types::TransactionSummary,
};

impl WalletService {
    pub fn send<N: NodeBackend>(
        &mut self,
        node: &N,
        destination: &str,
        amount: Amount,
        fee_rate: FeeRate,
    ) -> WalletResult<TransactionSummary> {
        let unchecked =
            Address::from_str(destination).map_err(|error| WalletError::InvalidAddress {
                address: destination.to_owned(),
                reason: error.to_string(),
            })?;
        let address = unchecked
            .require_network(self.wallet.network())
            .map_err(|_| WalletError::AddressNetworkMismatch {
                address: destination.to_owned(),
                expected: self.wallet.network(),
            })?;

        let mut builder = self.wallet.build_tx();
        builder.add_recipient(address.script_pubkey(), amount);
        builder.fee_rate(fee_rate);

        let mut psbt = builder.finish()?;
        let finalized = self.wallet.sign(&mut psbt, SignOptions::default())?;
        if !finalized {
            return Err(WalletError::TransactionNotFinalized);
        }

        let fee = psbt
            .fee_amount()
            .ok_or(WalletError::MissingTransactionFee)?;
        let transaction = psbt
            .extract_tx()
            .map_err(|error| WalletError::ExtractTransaction(error.to_string()))?;

        // Persist a newly revealed change index before broadcasting so it cannot
        // be reused if the process exits immediately after the RPC call.
        self.wallet.persist(self.store.connection_mut())?;

        let txid = node.broadcast(&transaction)?;
        let now = UNIX_EPOCH.elapsed().unwrap_or_default().as_secs();
        self.wallet.apply_unconfirmed_txs([(transaction, now)]);
        self.wallet.persist(self.store.connection_mut())?;

        Ok(TransactionSummary {
            txid,
            sent: amount,
            fee,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use bdk_wallet::KeychainKind;
    use bitcoin::{
        Amount, FeeRate, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
        Witness, absolute, hashes::Hash, transaction,
    };
    use tempfile::tempdir;

    use super::WalletService;
    use crate::{
        config::{RpcConfig, WalletConfig},
        error::{WalletError, WalletResult},
        node::NodeBackend,
    };

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[derive(Default)]
    struct RecordingNode {
        transactions: RefCell<Vec<Transaction>>,
    }

    impl NodeBackend for RecordingNode {
        fn tip_height(&self) -> WalletResult<u64> {
            Ok(0)
        }

        fn broadcast(&self, transaction: &Transaction) -> WalletResult<Txid> {
            self.transactions.borrow_mut().push(transaction.clone());
            Ok(transaction.compute_txid())
        }
    }

    fn config(data_dir: std::path::PathBuf) -> WalletConfig {
        WalletConfig::new(
            Network::Regtest,
            data_dir,
            RpcConfig::new("http://127.0.0.1:18443".into(), None, None),
        )
        .unwrap()
    }

    fn add_spendable_change(service: &mut WalletService, value: Amount) {
        let address = service
            .wallet
            .reveal_next_address(KeychainKind::Internal)
            .address;
        let funding = Transaction {
            version: transaction::Version::TWO,
            lock_time: absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::new(Txid::from_byte_array([1; 32]), 0),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value,
                script_pubkey: address.script_pubkey(),
            }],
        };

        service.wallet.apply_unconfirmed_txs([(funding, 1)]);
    }

    #[test]
    fn rejects_invalid_and_wrong_network_addresses() {
        let temp = tempdir().unwrap();
        let config = config(temp.path().join("wallet"));
        let (mut service, _) = WalletService::initialize(&config, Some(MNEMONIC)).unwrap();
        let node = RecordingNode::default();
        let fee_rate = FeeRate::from_sat_per_vb(2).unwrap();

        let invalid = service.send(
            &node,
            "definitely-not-an-address",
            Amount::from_sat(10_000),
            fee_rate,
        );
        let wrong_network = service.send(
            &node,
            "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
            Amount::from_sat(10_000),
            fee_rate,
        );

        assert!(matches!(invalid, Err(WalletError::InvalidAddress { .. })));
        assert!(matches!(
            wrong_network,
            Err(WalletError::AddressNetworkMismatch { .. })
        ));
        assert!(node.transactions.borrow().is_empty());
    }

    #[test]
    fn builds_signs_broadcasts_and_persists_a_transaction() {
        let temp = tempdir().unwrap();
        let config = config(temp.path().join("wallet"));
        let (mut service, _) = WalletService::initialize(&config, Some(MNEMONIC)).unwrap();
        add_spendable_change(&mut service, Amount::from_sat(100_000));
        let destination = service
            .wallet
            .reveal_next_address(KeychainKind::External)
            .address
            .to_string();
        let node = RecordingNode::default();

        let summary = service
            .send(
                &node,
                &destination,
                Amount::from_sat(40_000),
                FeeRate::from_sat_per_vb(2).unwrap(),
            )
            .unwrap();

        assert_eq!(summary.sent, Amount::from_sat(40_000));
        assert!(summary.fee > Amount::ZERO);
        let transactions = node.transactions.borrow();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].compute_txid(), summary.txid);
        assert!(
            transactions[0]
                .input
                .iter()
                .all(|input| !input.witness.is_empty())
        );
        drop(transactions);
        drop(service);

        let reopened = WalletService::load(&config, None).unwrap();
        assert!(reopened.wallet.get_tx(summary.txid).is_some());
    }
}
