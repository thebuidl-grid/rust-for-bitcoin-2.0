use anyhow::{Context, Result};
use bdk_bitcoind_rpc::bitcoincore_rpc::bitcoincore_rpc_json::GetBlockchainInfoResult;
use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client, RpcApi};
use bdk_bitcoind_rpc::Emitter;
use bdk_wallet::bitcoin::{Transaction, Txid};
use std::sync::Arc;

use crate::wallet::WalletManager;

pub struct NodeClient {
    pub client: Arc<Client>,
}

impl NodeClient {
    /// Connects to a Bitcoin Core node via RPC.
    pub fn new(url: &str, user: Option<String>, pass: Option<String>) -> Result<Self> {
        let auth = match (user, pass) {
            (Some(u), Some(p)) => Auth::UserPass(u, p),
            _ => Auth::None,
        };

        let client = Client::new(url, auth)
            .with_context(|| format!("Failed to connect to Bitcoin Core RPC at {}", url))?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Syncs the wallet state block-by-block and mempool updates from Bitcoin Core.
    pub fn sync(&self, wallet_mgr: &mut WalletManager, start_height: u32) -> Result<usize> {
        let wallet_tip = wallet_mgr.wallet.latest_checkpoint();

        let mut emitter = Emitter::new(
            self.client.as_ref(),
            wallet_tip,
            start_height,
            wallet_mgr
                .wallet
                .transactions()
                .filter(|tx| tx.chain_position.is_unconfirmed()),
        );

        let mut blocks_applied = 0;
        while let Some(block_emission) = emitter.next_block()? {
            let height = block_emission.block_height();
            let connected_to = block_emission.connected_to();
            
            wallet_mgr.wallet.apply_block_connected_to(
                &block_emission.block,
                height,
                connected_to,
            )?;
            blocks_applied += 1;
        }

        let mempool_event = emitter.mempool()?;
        wallet_mgr.wallet.apply_evicted_txs(mempool_event.evicted);
        wallet_mgr.wallet.apply_unconfirmed_txs(mempool_event.update);

        wallet_mgr.save()?;

        Ok(blocks_applied)
    }

    /// Broadcast a signed raw transaction to the Bitcoin network.
    pub fn broadcast(&self, tx: &Transaction) -> Result<Txid> {
        let txid = self.client.send_raw_transaction(tx)
            .context("Failed to broadcast transaction via RPC")?;
        Ok(txid)
    }

    /// Get general blockchain info from connected node.
    pub fn get_info(&self) -> Result<GetBlockchainInfoResult> {
        Ok(self.client.get_blockchain_info()?)
    }
}
