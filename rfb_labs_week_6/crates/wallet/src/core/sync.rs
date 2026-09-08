use bdk_bitcoind_rpc::Emitter;
use bdk_wallet::KeychainKind;
use bitcoin::constants::COINBASE_MATURITY;

use crate::{
    core::WalletService,
    error::WalletResult,
    node::BitcoinCoreNode,
    types::{Keychain, WalletBalance, WalletSync, WalletUtxo},
};

impl WalletService {
    pub fn sync(&mut self, node: &BitcoinCoreNode) -> WalletResult<WalletSync> {
        let checkpoint = self.wallet.latest_checkpoint();
        let unconfirmed = self
            .wallet
            .transactions()
            .filter(|transaction| transaction.chain_position.is_unconfirmed());
        let mut emitter = Emitter::new(node.rpc_client(), checkpoint, 0, unconfirmed);
        let mut blocks_applied = 0;

        while let Some(emission) = emitter
            .next_block()
            .map_err(|error| node.connection_error(error))?
        {
            self.wallet.apply_block_connected_to(
                &emission.block,
                emission.block_height(),
                emission.connected_to(),
            )?;
            self.wallet.persist(self.store.connection_mut())?;
            blocks_applied += 1;
        }

        let mempool = emitter
            .mempool()
            .map_err(|error| node.connection_error(error))?;
        let mempool_transactions = mempool.update.len();
        let evicted_transactions = mempool.evicted.len();
        self.wallet.apply_evicted_txs(mempool.evicted);
        self.wallet.apply_unconfirmed_txs(mempool.update);
        self.wallet.persist(self.store.connection_mut())?;

        let tip = self.wallet.latest_checkpoint();

        Ok(WalletSync {
            blocks_applied,
            mempool_transactions,
            evicted_transactions,
            tip_height: tip.height(),
            tip_hash: tip.hash(),
        })
    }

    pub fn balance(&self) -> WalletResult<WalletBalance> {
        let balance = self.wallet.balance();
        let spendable = self
            .list_utxos()?
            .into_iter()
            .filter(|utxo| utxo.spendable)
            .fold(bitcoin::Amount::ZERO, |total, utxo| total + utxo.value);

        Ok(WalletBalance {
            confirmed: balance.confirmed,
            trusted_pending: balance.trusted_pending,
            untrusted_pending: balance.untrusted_pending,
            pending: balance.trusted_pending + balance.untrusted_pending,
            immature: balance.immature,
            spendable,
            total: balance.total(),
        })
    }

    pub fn list_utxos(&self) -> WalletResult<Vec<WalletUtxo>> {
        let tip_height = self.wallet.latest_checkpoint().height();

        Ok(self
            .wallet
            .list_unspent()
            .map(|output| {
                let confirmed = output.chain_position.is_confirmed();
                let confirmation_height = output.chain_position.confirmation_height_upper_bound();
                let coinbase = self
                    .wallet
                    .get_tx(output.outpoint.txid)
                    .is_some_and(|transaction| transaction.tx_node.tx.is_coinbase());
                let confirmations = confirmation_height
                    .map(|height| tip_height.saturating_sub(height) + 1)
                    .unwrap_or(0);
                let mature = !coinbase || confirmations >= COINBASE_MATURITY;
                let keychain = match output.keychain {
                    KeychainKind::External => Keychain::External,
                    KeychainKind::Internal => Keychain::Internal,
                };
                let locked = self.wallet.is_outpoint_locked(output.outpoint);
                let trusted = keychain == Keychain::Internal;

                WalletUtxo {
                    outpoint: output.outpoint,
                    value: output.txout.value,
                    keychain,
                    derivation_index: output.derivation_index,
                    confirmation_height,
                    confirmed,
                    coinbase,
                    mature,
                    locked,
                    spendable: !locked && mature && (confirmed || trusted),
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use bdk_wallet::KeychainKind;
    use bitcoin::{
        Amount, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
        absolute, hashes::Hash, transaction,
    };
    use tempfile::tempdir;

    use super::WalletService;
    use crate::{
        config::{RpcConfig, WalletConfig},
        types::Keychain,
    };

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn config(data_dir: std::path::PathBuf) -> WalletConfig {
        WalletConfig::new(
            Network::Regtest,
            data_dir,
            RpcConfig::new("http://127.0.0.1:18443".into(), None, None),
        )
        .unwrap()
    }

    fn receive_unconfirmed(
        service: &mut WalletService,
        keychain: KeychainKind,
        value: Amount,
        previous_tx_tag: u8,
    ) {
        let address = service.wallet.reveal_next_address(keychain).address;
        let transaction = Transaction {
            version: transaction::Version::TWO,
            lock_time: absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::new(Txid::from_byte_array([previous_tx_tag; 32]), 0),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value,
                script_pubkey: address.script_pubkey(),
            }],
        };

        service.wallet.apply_unconfirmed_txs([(transaction, 1)]);
    }

    #[test]
    fn empty_wallet_has_zero_balance_and_no_utxos() {
        let temp = tempdir().unwrap();
        let config = config(temp.path().join("wallet"));
        let (service, _) = WalletService::initialize(&config, Some(MNEMONIC)).unwrap();

        let balance = service.balance().unwrap();

        assert_eq!(balance.total, Amount::ZERO);
        assert_eq!(balance.spendable, Amount::ZERO);
        assert!(service.list_utxos().unwrap().is_empty());
    }

    #[test]
    fn classifies_and_persists_pending_receiving_and_change_outputs() {
        let temp = tempdir().unwrap();
        let config = config(temp.path().join("wallet"));
        let (mut service, _) = WalletService::initialize(&config, Some(MNEMONIC)).unwrap();
        receive_unconfirmed(
            &mut service,
            KeychainKind::External,
            Amount::from_sat(50_000),
            1,
        );
        receive_unconfirmed(
            &mut service,
            KeychainKind::Internal,
            Amount::from_sat(20_000),
            2,
        );
        service
            .wallet
            .persist(service.store.connection_mut())
            .unwrap();
        drop(service);

        let mut reopened = WalletService::load(&config, None).unwrap();
        let balance = reopened.balance().unwrap();
        let mut utxos = reopened.list_utxos().unwrap();
        utxos.sort_by_key(|utxo| utxo.value);

        assert_eq!(balance.confirmed, Amount::ZERO);
        assert_eq!(balance.trusted_pending, Amount::from_sat(20_000));
        assert_eq!(balance.untrusted_pending, Amount::from_sat(50_000));
        assert_eq!(balance.pending, Amount::from_sat(70_000));
        assert_eq!(balance.spendable, Amount::from_sat(20_000));
        assert_eq!(balance.total, Amount::from_sat(70_000));
        assert_eq!(utxos.len(), 2);

        assert_eq!(utxos[0].keychain, Keychain::Internal);
        assert!(!utxos[0].confirmed);
        assert!(utxos[0].mature);
        assert!(utxos[0].spendable);

        assert_eq!(utxos[1].keychain, Keychain::External);
        assert!(!utxos[1].confirmed);
        assert!(utxos[1].mature);
        assert!(!utxos[1].spendable);

        reopened.wallet.lock_outpoint(utxos[0].outpoint);

        assert_eq!(reopened.balance().unwrap().spendable, Amount::ZERO);
        assert!(
            reopened
                .list_utxos()
                .unwrap()
                .into_iter()
                .find(|utxo| utxo.outpoint == utxos[0].outpoint)
                .unwrap()
                .locked
        );
    }
}
