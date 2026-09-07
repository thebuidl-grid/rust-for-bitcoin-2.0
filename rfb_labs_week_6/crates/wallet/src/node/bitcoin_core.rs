use bitcoin::{Transaction, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::{config::RpcConfig, error::WalletResult};

/// Narrow interface used by the wallet layer for chain operations.
///
/// Keeping this trait small makes the wallet service testable without a running
/// Bitcoin Core node.
pub trait NodeBackend {
    fn tip_height(&self) -> WalletResult<u64>;
    fn broadcast(&self, transaction: &Transaction) -> WalletResult<Txid>;
}

pub struct BitcoinCoreNode {
    client: Client,
}

impl BitcoinCoreNode {
    pub fn connect(config: &RpcConfig) -> WalletResult<Self> {
        let auth = match (&config.user, &config.password) {
            (Some(user), Some(password)) => Auth::UserPass(user.clone(), password.clone()),
            _ => Auth::None,
        };

        Ok(Self {
            client: Client::new(&config.url, auth)?,
        })
    }
}

impl NodeBackend for BitcoinCoreNode {
    fn tip_height(&self) -> WalletResult<u64> {
        Ok(self.client.get_block_count()?)
    }

    fn broadcast(&self, transaction: &Transaction) -> WalletResult<Txid> {
        Ok(self.client.send_raw_transaction(transaction)?)
    }
}
