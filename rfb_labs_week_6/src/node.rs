//! Bitcoin Core connection and chain synchronisation.
//!
//! # What the node is used for, and what it is not
//!
//! This wallet does **not** use Bitcoin Core's wallet. No `createwallet`, no
//! `importdescriptors`, no `getbalance`. Core does exactly two jobs:
//!
//! 1. **Chain source** — hand over blocks so we can find our own outputs.
//! 2. **Broadcaster** — relay a signed transaction (Phase 3).
//!
//! Everything else — key derivation, address generation, UTXO tracking, balance,
//! coin selection, signing — happens in this process through BDK. That is the point
//! of a descriptor wallet: the node never learns who we are.
//!
//! # How the sync loop works
//!
//! [`bdk_bitcoind_rpc::Emitter`] walks the chain from a checkpoint and yields blocks
//! that connect to what the wallet already knows. Each [`BlockEvent`] carries a
//! `connected_to` [`BlockId`], which is how BDK detects and unwinds reorgs rather
//! than blindly appending.
//!
//! After the blocks, the mempool is polled so freshly broadcast transactions show up
//! as pending balance before they confirm.

use bdk_bitcoind_rpc::Emitter;
use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client, RpcApi};
use bdk_wallet::bitcoin::{Address, BlockHash, Network};

use crate::config::Config;
use crate::error::{Result, WalletError};
use crate::wallet::Wallet;

/// Outcome of a sync run, for reporting.
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncReport {
    pub blocks_applied: u32,
    pub tip_height: u32,
    pub mempool_txs: usize,
    pub evicted_txs: usize,
}

/// Connect to the configured node.
///
/// Polar authenticates with `-rpcauth`, which means no cookie file is written, so
/// user/password is the only option here.
pub fn connect(config: &Config) -> Result<Client> {
    let client = Client::new(
        &config.rpc.url,
        Auth::UserPass(config.rpc.user.clone(), config.rpc.password.clone()),
    )?;

    // Fail fast and clearly if the node is on a different chain than we expect.
    // Without this the wallet would sync happily and derive addresses that nobody
    // on that network can pay.
    let info = client.get_blockchain_info()?;
    let node_network = core_chain_to_network(&info.chain.to_string());
    if node_network != Some(config.network) {
        return Err(WalletError::NetworkMismatch {
            node: info.chain.to_string(),
            wallet: config.network.to_string(),
        });
    }

    Ok(client)
}

/// Bring the wallet up to date with the node.
///
/// Blocks first, then the mempool. Persist happens once at the end rather than per
/// block: a changeset is a set of deltas, so one flush of many blocks is both
/// correct and far faster than a write per block.
pub fn sync(wallet: &mut Wallet, client: &Client) -> Result<SyncReport> {
    let mut report = SyncReport::default();

    // Resume from where we left off. On a fresh wallet this is the genesis
    // checkpoint, so the first sync walks the whole (short) regtest chain.
    let checkpoint = wallet.inner().latest_checkpoint();
    let start_height = checkpoint.height();

    // Seed the emitter with the unconfirmed transactions we already know about, so
    // it can tell us if any were evicted from the mempool while we were away.
    let expected: Vec<_> = wallet
        .inner()
        .transactions()
        .filter(|tx| !tx.chain_position.is_confirmed())
        .map(|tx| tx.tx_node.tx.clone())
        .collect();

    let mut emitter = Emitter::new(client, checkpoint, start_height, expected);

    while let Some(event) = emitter.next_block()? {
        let height = event.block_height();
        // `connected_to` is what lets BDK handle reorgs: it names the block this one
        // builds on, so a fork is detected instead of silently appended.
        wallet
            .inner_mut()
            .apply_block_connected_to(&event.block, height, event.connected_to())?;

        report.blocks_applied += 1;
        report.tip_height = height;
    }

    // Unconfirmed transactions, so a just-broadcast payment shows as pending.
    let mempool = emitter.mempool()?;
    report.mempool_txs = mempool.update.len();
    report.evicted_txs = mempool.evicted.len();

    wallet.inner_mut().apply_evicted_txs(mempool.evicted);
    wallet.inner_mut().apply_unconfirmed_txs(mempool.update);

    wallet.persist()?;

    if report.tip_height == 0 {
        report.tip_height = wallet.inner().latest_checkpoint().height();
    }

    Ok(report)
}

/// Mine `blocks` blocks paying `address`. Regtest only.
///
/// Coinbase outputs need 100 confirmations before they can be spent, so mining a
/// single block leaves the funds `immature`. Mining 101 matures exactly the first
/// one. On regtest the subsidy stays at 50 BTC for the first 150 blocks.
pub fn mine_to(client: &Client, address: &Address, blocks: u64) -> Result<Vec<BlockHash>> {
    Ok(client.generate_to_address(blocks, address)?)
}

/// The node's current block height.
pub fn tip_height(client: &Client) -> Result<u64> {
    Ok(client.get_block_count()?)
}

/// Map Core's chain name onto a `Network`.
///
/// `getblockchaininfo` reports `main` / `test` / `testnet4` / `signet` / `regtest`,
/// which is a different vocabulary from `Network`'s `FromStr` (`bitcoin`,
/// `testnet`, ...). `Network::from_core_arg` speaks the former.
fn core_chain_to_network(chain: &str) -> Option<Network> {
    Network::from_core_arg(chain)
        .ok()
        // Older/newer Core versions have used both spellings, so fall back.
        .or_else(|| chain.parse::<Network>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_core_chain_names() {
        assert_eq!(core_chain_to_network("regtest"), Some(Network::Regtest));
        assert_eq!(core_chain_to_network("signet"), Some(Network::Signet));
        assert_eq!(core_chain_to_network("main"), Some(Network::Bitcoin));
        // `getblockchaininfo` says "test" where `Network::from_str` says "testnet";
        // both must resolve, which is why the fallback exists.
        assert_eq!(core_chain_to_network("test"), Some(Network::Testnet));
        assert_eq!(core_chain_to_network("testnet"), Some(Network::Testnet));
        assert_eq!(core_chain_to_network("nonsense"), None);
    }
}
