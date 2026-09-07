//! Everything that talks to `bitcoind` over JSON-RPC.
//!
//! This is plain `bitcoincore-rpc`, not the node's own wallet RPCs. The node is
//! used purely as a chain source and a relay: `getblockchaininfo` to check we
//! are on the chain we think we are, block fetches for the sync emitter,
//! `sendrawtransaction` to broadcast, and `generatetoaddress` to make regtest
//! coins. A wallet-disabled node (`-disablewallet`) serves all of it.

use bitcoin::{Address, Amount, BlockHash, Network, Transaction, Txid};
use bitcoincore_rpc::{Client, RpcApi};

use crate::config::Config;
use crate::error::{Error, Result};

/// A snapshot of the node, used by the `node` command and by sync.
#[derive(Debug)]
pub struct NodeInfo {
    pub chain: Network,
    pub blocks: u64,
    pub headers: u64,
    pub best_block_hash: BlockHash,
    pub initial_block_download: bool,
    pub subversion: String,
    pub connections: usize,
}

/// Connect and confirm the node is serving the network we are configured for.
///
/// Doing this once at startup turns a whole class of confusing downstream
/// failures ("why is my balance zero?") into one clear message.
pub fn connect(cfg: &Config) -> Result<Client> {
    let client = Client::new(&cfg.rpc_url, cfg.rpc_auth.clone().into_auth())?;
    let info = client.get_blockchain_info()?;
    if info.chain != cfg.network {
        return Err(Error::NetworkMismatch {
            node: info.chain,
            configured: cfg.network,
        });
    }
    Ok(client)
}

pub fn info(client: &Client) -> Result<NodeInfo> {
    let chain = client.get_blockchain_info()?;
    let net = client.get_network_info()?;
    Ok(NodeInfo {
        chain: chain.chain,
        blocks: chain.blocks,
        headers: chain.headers,
        best_block_hash: chain.best_block_hash,
        initial_block_download: chain.initial_block_download,
        subversion: net.subversion,
        connections: net.connections,
    })
}

pub fn block_count(client: &Client) -> Result<u32> {
    Ok(client.get_block_count()? as u32)
}

/// Relay a fully signed transaction.
pub fn broadcast(client: &Client, tx: &Transaction) -> Result<Txid> {
    Ok(client.send_raw_transaction(tx)?)
}

/// Mine blocks straight to `address`. Regtest only, and guarded as such.
pub fn mine_to(
    client: &Client,
    network: Network,
    address: &Address,
    blocks: u64,
) -> Result<Vec<BlockHash>> {
    if network != Network::Regtest {
        return Err(Error::RegtestOnly("mine"));
    }
    Ok(client.generate_to_address(blocks, address)?)
}

/// Fetch a raw transaction from the node.
///
/// Returns `Ok(None)` rather than an error when the node simply does not have
/// it, which is the normal case for a confirmed transaction on a node without
/// `-txindex`.
pub fn raw_transaction(client: &Client, txid: &Txid) -> Result<Option<Transaction>> {
    match client.get_raw_transaction(txid, None) {
        Ok(tx) => Ok(Some(tx)),
        Err(e) if is_not_found(&e) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// A rough fee estimate, falling back to a sane regtest floor.
///
/// `estimatesmartfee` has nothing to work with on a fresh regtest chain, so a
/// missing estimate is expected rather than exceptional.
pub fn fee_estimate(client: &Client, target_blocks: u16) -> Option<Amount> {
    client
        .estimate_smart_fee(target_blocks, None)
        .ok()
        .and_then(|r| r.fee_rate)
}

fn is_not_found(err: &bitcoincore_rpc::Error) -> bool {
    matches!(
        err,
        bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Rpc(rpc))
            if rpc.code == -5
    )
}
