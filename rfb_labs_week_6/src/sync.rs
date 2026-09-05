//! Connect to Bitcoin Core over RPC and sync a `bdk_wallet::Wallet` against
//! it: walk new blocks since the wallet's last checkpoint, then apply the
//! current mempool, persisting after each step.

use anyhow::{Context, Result, bail};
use bdk_bitcoind_rpc::Emitter;
use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client, RpcApi};
use bdk_wallet::rusqlite::Connection;

use crate::config::Config;
use crate::descriptors::Wallet;

/// Build an RPC client from `config`, preferring a cookie file, then
/// user/pass, then no auth at all (only appropriate for a fully open node).
pub fn rpc_client(config: &Config) -> Result<Client> {
    let auth = match (&config.rpc_cookie, &config.rpc_user, &config.rpc_pass) {
        (Some(cookie), _, _) => Auth::CookieFile(cookie.clone()),
        (None, Some(user), Some(pass)) => Auth::UserPass(user.clone(), pass.clone()),
        (None, None, None) => Auth::None,
        _ => bail!("set both RPC_USER and RPC_PASS together, or RPC_COOKIE, or neither"),
    };
    let client = Client::new(&config.rpc_url, auth)
        .with_context(|| format!("failed to reach Bitcoin Core RPC at {}", config.rpc_url))?;
    // Fail fast with a clear error rather than deep inside the emitter.
    client
        .get_blockchain_info()
        .context("Bitcoin Core RPC did not respond to getblockchaininfo")?;
    Ok(client)
}

pub struct SyncSummary {
    pub blocks_applied: usize,
    pub tip_height: u32,
}

/// Sync `wallet` against `client`: apply every block connected since the
/// wallet's last checkpoint, then the current mempool, persisting to `db`
/// after each stage so a crash mid-sync loses at most one step.
pub fn sync_wallet(
    wallet: &mut Wallet,
    db: &mut Connection,
    client: &Client,
) -> Result<SyncSummary> {
    let wallet_tip = wallet.latest_checkpoint();
    let mut emitter = Emitter::new(
        client,
        wallet_tip,
        0,
        wallet
            .transactions()
            .filter(|tx| tx.chain_position.is_unconfirmed()),
    );

    let mut blocks_applied = 0usize;
    while let Some(block_emission) = emitter.next_block().context("failed to fetch next block")? {
        let height = block_emission.block_height();
        let connected_to = block_emission.connected_to();
        wallet
            .apply_block_connected_to(&block_emission.block, height, connected_to)
            .context("failed to apply a synced block to the wallet")?;
        wallet
            .persist(db)
            .context("failed to persist wallet state after a block")?;
        blocks_applied += 1;
    }

    let mempool_event = emitter.mempool().context("failed to fetch the mempool")?;
    wallet.apply_evicted_txs(mempool_event.evicted);
    wallet.apply_unconfirmed_txs(mempool_event.update);
    wallet
        .persist(db)
        .context("failed to persist wallet state after the mempool")?;

    Ok(SyncSummary {
        blocks_applied,
        tip_height: wallet.latest_checkpoint().height(),
    })
}
