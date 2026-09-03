//! Everything that talks to `bitcoind`.
//!
//! We connect via `bdk_bitcoind_rpc`, which re-exports `bitcoincore-rpc` as
//! `bdk_bitcoind_rpc::bitcoincore_rpc` and adds an [`Emitter`] that turns the node's raw block/
//! mempool data into updates BDK's `Wallet` can apply. Using the re-export (rather than a second,
//! independently-versioned `bitcoincore-rpc` dependency) guarantees our RPC types match the ones
//! `Emitter` expects.

use anyhow::{Context, Result};
use bdk_bitcoind_rpc::Emitter;
use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client, RpcApi};
use bdk_wallet::rusqlite::Connection;

use crate::config::Config;
use crate::wallet_store::Wallet;

/// Build an RPC client from `Config`, preferring cookie auth, falling back to user/pass, and
/// finally to no auth (only sensible for a locally trusted regtest node).
pub fn connect(cfg: &Config) -> Result<Client> {
    let auth = match (&cfg.rpc_cookie, &cfg.rpc_user, &cfg.rpc_pass) {
        (Some(path), _, _) => Auth::CookieFile(path.clone()),
        (None, Some(user), Some(pass)) => Auth::UserPass(user.clone(), pass.clone()),
        (None, Some(_), None) => anyhow::bail!("RPC_USER is set but RPC_PASS is missing"),
        (None, None, Some(_)) => anyhow::bail!("RPC_PASS is set but RPC_USER is missing"),
        (None, None, None) => Auth::None,
    };
    let client = Client::new(&cfg.rpc_url, auth)
        .with_context(|| format!("failed to build an RPC client for {}", cfg.rpc_url))?;
    client.get_blockchain_info().with_context(|| {
        format!(
            "could not reach bitcoind at {} — is it running?",
            cfg.rpc_url
        )
    })?;
    Ok(client)
}

/// Walk the chain block-by-block from the wallet's last-known tip, apply every relevant block and
/// the current mempool snapshot to the wallet, and persist after each step so a crash mid-sync
/// never loses already-applied blocks.
pub fn sync_wallet(
    wallet: &mut Wallet,
    db: &mut Connection,
    client: &Client,
    start_height: u32,
) -> Result<usize> {
    let wallet_tip = wallet.latest_checkpoint();
    let mut emitter = Emitter::new(
        client,
        wallet_tip,
        start_height,
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
            .context("failed to apply block to wallet")?;
        wallet
            .persist(db)
            .context("failed to persist wallet state")?;
        blocks_applied += 1;
    }

    let mempool = emitter.mempool().context("failed to fetch mempool")?;
    wallet.apply_evicted_txs(mempool.evicted);
    wallet.apply_unconfirmed_txs(mempool.update);
    wallet
        .persist(db)
        .context("failed to persist wallet state")?;

    Ok(blocks_applied)
}
