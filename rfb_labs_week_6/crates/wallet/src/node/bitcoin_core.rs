use std::sync::Arc;

use bitcoin::{Network, Transaction, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::{
    config::RpcConfig,
    error::{WalletError, WalletResult},
};

/// Narrow interface used by the wallet layer for chain operations.
///
/// Keeping this trait small makes the wallet service testable without a running
/// Bitcoin Core node.
pub trait NodeBackend {
    fn tip_height(&self) -> WalletResult<u64>;
    fn broadcast(&self, transaction: &Transaction) -> WalletResult<Txid>;
}

pub struct BitcoinCoreNode {
    client: Arc<Client>,
    url: String,
}

impl BitcoinCoreNode {
    /// Connects to Bitcoin Core and verifies that it serves the expected chain.
    pub fn connect(config: &RpcConfig, expected_network: Network) -> WalletResult<Self> {
        let auth = match (&config.user, &config.password) {
            (Some(user), Some(password)) => Auth::UserPass(user.clone(), password.clone()),
            _ => Auth::None,
        };

        let client = Client::new(&config.url, auth).map_err(|source| {
            WalletError::BitcoinCoreConnection {
                url: config.url.clone(),
                source,
            }
        })?;
        let info =
            client
                .get_blockchain_info()
                .map_err(|source| WalletError::BitcoinCoreConnection {
                    url: config.url.clone(),
                    source,
                })?;

        ensure_expected_network(&config.url, expected_network, info.chain)?;
        tracing::debug!(
            url = %config.url,
            network = %info.chain,
            height = info.blocks,
            "Bitcoin Core health check passed"
        );

        Ok(Self {
            client: Arc::new(client),
            url: config.url.clone(),
        })
    }

    pub(crate) fn rpc_client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }

    pub(crate) fn connection_error(&self, source: bitcoincore_rpc::Error) -> WalletError {
        WalletError::BitcoinCoreConnection {
            url: self.url.clone(),
            source,
        }
    }
}

fn ensure_expected_network(url: &str, expected: Network, actual: Network) -> WalletResult<()> {
    if actual != expected {
        return Err(WalletError::BitcoinCoreNetworkMismatch {
            url: url.to_owned(),
            expected,
            actual,
        });
    }

    Ok(())
}

impl NodeBackend for BitcoinCoreNode {
    fn tip_height(&self) -> WalletResult<u64> {
        Ok(self.client.get_block_count()?)
    }

    fn broadcast(&self, transaction: &Transaction) -> WalletResult<Txid> {
        Ok(self.client.send_raw_transaction(transaction)?)
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::Network;

    use super::ensure_expected_network;
    use crate::error::WalletError;

    #[test]
    fn accepts_the_expected_node_network() {
        ensure_expected_network("http://127.0.0.1:18443", Network::Regtest, Network::Regtest)
            .unwrap();
    }

    #[test]
    fn rejects_a_node_on_the_wrong_network() {
        let result =
            ensure_expected_network("http://127.0.0.1:18443", Network::Regtest, Network::Bitcoin);

        assert!(matches!(
            result,
            Err(WalletError::BitcoinCoreNetworkMismatch {
                expected: Network::Regtest,
                actual: Network::Bitcoin,
                ..
            })
        ));
    }
}
