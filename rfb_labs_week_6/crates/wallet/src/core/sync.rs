use bdk_bitcoind_rpc::Emitter;

use crate::{
    core::WalletService,
    error::{WalletError, WalletResult},
    node::BitcoinCoreNode,
    types::{WalletBalance, WalletSync, WalletUtxo},
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
        Err(WalletError::NotImplemented("balance"))
    }

    pub fn list_utxos(&self) -> WalletResult<Vec<WalletUtxo>> {
        Err(WalletError::NotImplemented("UTXO listing"))
    }
}
