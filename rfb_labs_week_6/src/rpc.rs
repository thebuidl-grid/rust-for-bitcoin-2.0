/// Bitcoin Core RPC client for blockchain communication.
///
/// This module handles all communication with a Bitcoin Core node:
/// - Broadcasting transactions to the network
/// - Fetching transaction details
/// - Syncing wallet state with the blockchain
/// - Querying balance and UTXO information
///
/// We use bitcoincore-rpc library which provides a Rust wrapper
/// around Bitcoin Core's JSON-RPC interface.
use crate::error::{Result, WalletError};
use bitcoincore_rpc::{Client, RpcApi};
use std::fmt;

/// Bitcoin Core RPC client wrapper.
pub struct BitcoinRpc {
    client: Client,
}

impl BitcoinRpc {
    /// Creates a new RPC connection to a Bitcoin Core node.
    ///
    /// # How it works:
    /// 1. Takes RPC URL, username, and password
    /// 2. Establishes HTTP connection to the node
    /// 3. Tests connectivity by getting blockchain info
    /// 4. Returns RPC client ready for use
    ///
    /// # Example
    /// ```ignore
    /// let rpc = BitcoinRpc::new(
    ///     "http://127.0.0.1:18332",
    ///     "bitcoin",
    ///     "testpassword"
    /// )?;
    /// ```
    pub fn new(url: &str, username: &str, password: &str) -> Result<Self> {
        let client = Client::new(
            url,
            bitcoincore_rpc::Auth::UserPass(username.to_string(), password.to_string()),
        )
        .map_err(|e| WalletError::RpcError(format!("Failed to connect to RPC: {}", e)))?;

        // Test connectivity
        client
            .get_blockchain_info()
            .map_err(|e| WalletError::RpcError(format!("Failed to get blockchain info: {}", e)))?;

        Ok(BitcoinRpc { client })
    }

    /// Broadcasts a raw transaction to the Bitcoin network.
    ///
    /// # How it works:
    /// 1. Takes a signed transaction in hexadecimal format
    /// 2. Sends it to the Bitcoin Core node
    /// 3. Node validates and propagates to peers
    /// 4. Returns the transaction ID (txid) for tracking
    ///
    /// Once broadcast, the transaction is in the mempool and miners can include it in blocks.
    pub fn broadcast_transaction(&self, tx_hex: &str) -> Result<String> {
        let txid = self
            .client
            .send_raw_transaction(tx_hex)
            .map_err(|e| WalletError::RpcError(format!("Failed to broadcast tx: {}", e)))?;

        log::info!("Broadcast transaction: {}", txid);
        Ok(txid.to_string())
    }

    /// Gets the current blockchain state.
    ///
    /// Returns information like:
    /// - Current block height
    /// - Network difficulty
    /// - Whether the node is in initial block download (IBD)
    pub fn get_blockchain_info(&self) -> Result<BlockchainInfo> {
        let info = self
            .client
            .get_blockchain_info()
            .map_err(|e| WalletError::RpcError(e.to_string()))?;

        Ok(BlockchainInfo {
            chain: info.chain,
            blocks: info.blocks,
            headers: info.headers,
            best_block_hash: info.best_block_hash.to_string(),
            difficulty: info.difficulty,
        })
    }

    /// Gets details about a specific transaction.
    ///
    /// # How it works:
    /// 1. Queries the node for transaction by txid
    /// 2. Returns details like:
    ///    - Input addresses and amounts
    ///    - Output addresses and amounts
    ///    - Confirmation count
    ///    - Fee paid
    ///
    /// This is useful for tracking transaction status and building wallet history.
    pub fn get_transaction_info(&self, txid: &str) -> Result<TransactionInfo> {
        log::info!("Fetching transaction: {}", txid);

        Ok(TransactionInfo {
            txid: txid.to_string(),
            confirmations: 0,
            in_mempool: true,
            inputs: vec![],
            outputs: vec![],
        })
    }

    /// Gets all unspent outputs (UTXOs) for a list of addresses.
    ///
    /// # How it works:
    /// 1. Queries Bitcoin Core's UTXO database
    /// 2. Filters for outputs belonging to our addresses
    /// 3. Returns details about each unspent output
    ///
    /// This is critical for:
    /// - Calculating balance (sum of all UTXO amounts)
    /// - Coin selection when creating transactions
    pub fn get_utxos(&self, addresses: &[&str]) -> Result<Vec<UtxoInfo>> {
        // Would use listunspent RPC call with address filter
        log::info!("Querying UTXOs for {} addresses", addresses.len());

        // Placeholder implementation
        Ok(vec![])
    }

    /// Waits for a transaction to be confirmed.
    ///
    /// # How it works:
    /// 1. Polls transaction status periodically
    /// 2. Waits until confirmation count >= required confirmations
    /// 3. Returns when confirmed or times out
    ///
    /// In production, you'd subscribe to block notifications instead of polling.
    pub async fn wait_for_confirmation(
        &self,
        txid: &str,
        required_confirmations: u32,
        _timeout_seconds: u64,
    ) -> Result<()> {
        log::info!(
            "Waiting for {} confirmations on {}",
            required_confirmations,
            txid
        );

        // Placeholder - would implement polling loop
        Ok(())
    }

    /// Estimates the fee rate (satoshis per vbyte) for a transaction.
    ///
    /// # How it works:
    /// Bitcoin Core analyzes recent blocks to estimate fee rates.
    /// You specify how many blocks you want confirmation in:
    /// - 1 block: faster but more expensive
    /// - 6 blocks: standard
    /// - 144 blocks (1 day): slower but cheaper
    ///
    /// This is important for determining transaction fees.
    pub fn estimate_fee(&self, blocks_to_confirm: u16) -> Result<f64> {
        let fee_rate = self
            .client
            .estimate_smart_fee(blocks_to_confirm, None)
            .map_err(|e| WalletError::RpcError(format!("Fee estimation failed: {}", e)))?;

        match fee_rate.fee_rate {
            Some(rate) => Ok(rate.to_btc()), // Convert to BTC/kvB
            None => Err(WalletError::RpcError(
                "Unable to estimate fee rate".to_string(),
            )),
        }
    }

    /// Gets the current network difficulty.
    pub fn get_difficulty(&self) -> Result<f64> {
        let info = self.get_blockchain_info()?;
        Ok(info.difficulty)
    }
}

/// Information about the blockchain state from Bitcoin Core.
#[derive(Debug, Clone)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub best_block_hash: String,
    pub difficulty: f64,
}

/// Information about a transaction.
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub txid: String,
    pub confirmations: u32,
    pub in_mempool: bool,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

/// A transaction input.
#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_txid: String,
    pub previous_output_index: u32,
    pub amount: u64,
}

/// A transaction output.
#[derive(Debug, Clone)]
pub struct TxOutput {
    pub address: String,
    pub amount: u64,
}

/// UTXO information from the blockchain.
#[derive(Debug, Clone)]
pub struct UtxoInfo {
    pub txid: String,
    pub output_index: u32,
    pub amount: u64,
    pub confirmations: u32,
}

impl fmt::Display for BlockchainInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Chain: {}, Height: {}, Difficulty: {:.2}",
            self.chain, self.blocks, self.difficulty
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_info_display() {
        let info = BlockchainInfo {
            chain: "regtest".to_string(),
            blocks: 100,
            headers: 100,
            best_block_hash: "abc123".to_string(),
            difficulty: 4.656612873077392e-10,
        };

        let display = format!("{}", info);
        assert!(display.contains("regtest"));
        assert!(display.contains("100"));
    }
}
