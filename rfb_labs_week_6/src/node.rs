use std::path::Path;
use std::sync::Arc;

use bdk_wallet::bitcoin::address::NetworkChecked;
use bdk_wallet::bitcoin::{Address, FeeRate, Transaction, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::config::{Config, RpcAuth};
use crate::error::WalletError;

/// A thin wrapper over `bitcoincore_rpc::Client`. This is the only place the
/// wallet talks directly to the node: connectivity checks, broadcasting, fee
/// estimation and (regtest only) block generation.
pub struct Node {
    client: Arc<Client>,
    network: bdk_wallet::bitcoin::Network,
}

impl Node {
    /// Connect and verify the node is reachable and on the expected network.
    pub fn connect(config: &Config) -> Result<Self, WalletError> {
        let auth = match &config.rpc_auth {
            RpcAuth::Cookie(path) => {
                if !Path::new(path).exists() {
                    return Err(WalletError::Connect(format!(
                        "RPC cookie file not found at {}",
                        path.display()
                    )));
                }
                Auth::CookieFile(path.clone())
            }
            RpcAuth::UserPass(user, pass) => Auth::UserPass(user.clone(), pass.clone()),
        };

        let client =
            Client::new(&config.rpc_url, auth).map_err(|e| WalletError::Connect(e.to_string()))?;

        let info = client
            .get_blockchain_info()
            .map_err(|e| WalletError::Connect(e.to_string()))?;

        if info.chain != config.network {
            return Err(WalletError::Connect(format!(
                "node is on '{}' but the wallet is configured for '{}'",
                info.chain, config.network
            )));
        }

        return Ok(Node {
            client: Arc::new(client),
            network: config.network,
        });
    }

    /// Shared handle for the block emitter (see `sync`).
    pub fn client(&self) -> Arc<Client> {
        return Arc::clone(&self.client);
    }

    pub fn blockchain_info(
        &self,
    ) -> Result<bitcoincore_rpc::json::GetBlockchainInfoResult, WalletError> {
        return Ok(self.client.get_blockchain_info()?);
    }

    /// Broadcast a fully signed transaction. Returns the txid the node accepted.
    pub fn broadcast(&self, tx: &Transaction) -> Result<Txid, WalletError> {
        return Ok(self.client.send_raw_transaction(tx)?);
    }

    /// Look up a confirmed or mempool transaction, proving a broadcast landed.
    pub fn raw_transaction_hex(&self, txid: &Txid) -> Result<String, WalletError> {
        let info = self.client.get_raw_transaction_info(txid, None)?;
        return Ok(info.hex.to_lowercase_hex());
    }

    /// Ask the node for a fee rate; fall back to 1 sat/vB when it has no
    /// estimate (which is always the case on a fresh regtest chain).
    pub fn fee_rate(&self) -> FeeRate {
        let fallback = FeeRate::from_sat_per_vb(1).expect("1 sat/vb is valid");
        let estimate = self.client.estimate_smart_fee(6, None);
        return match estimate {
            Ok(result) => match result.fee_rate {
                Some(per_kvb) => {
                    let per_vb = (per_kvb.to_sat() / 1000).max(1);
                    FeeRate::from_sat_per_vb(per_vb).unwrap_or(fallback)
                }
                None => fallback,
            },
            Err(_) => fallback,
        };
    }

    /// Mine `blocks` blocks to `address`. Regtest only.
    pub fn mine(
        &self,
        blocks: u64,
        address: &Address<NetworkChecked>,
    ) -> Result<Vec<String>, WalletError> {
        if self.network != bdk_wallet::bitcoin::Network::Regtest {
            return Err(WalletError::RegtestOnly);
        }
        let hashes = self.client.generate_to_address(blocks, address)?;
        return Ok(hashes.into_iter().map(|h| h.to_string()).collect());
    }
}

// Bring the lower-hex helper for `Vec<u8>` style hex without pulling another crate.
trait ToLowerHex {
    fn to_lowercase_hex(&self) -> String;
}

impl ToLowerHex for Vec<u8> {
    fn to_lowercase_hex(&self) -> String {
        let mut s = String::with_capacity(self.len() * 2);
        for byte in self {
            s.push_str(&format!("{byte:02x}"));
        }
        return s;
    }
}
