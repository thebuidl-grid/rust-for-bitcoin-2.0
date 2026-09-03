use std::sync::Arc;

use anyhow::{Context, Result};
use bdk_bitcoind_rpc::Emitter;
use bdk_bitcoind_rpc::bitcoincore_rpc::Client;
use bdk_wallet::bitcoin::Network;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, PersistedWallet, Wallet};

use crate::keys::DescriptorPair;

/// Open the wallet's SQLite store, loading existing state if present or creating it fresh.
///
/// The private descriptors (holding the signing keys, re-derived from the mnemonic on every
/// run) are supplied here but never written to the SQLite store itself -- only the resulting
/// public chain state (UTXOs, tx graph, keychain indices) is persisted. See the README for why.
pub fn open_or_create(
    db_path: &std::path::Path,
    descriptors: &DescriptorPair,
    network: Network,
) -> Result<(PersistedWallet<Connection>, Connection)> {
    let mut db = Connection::open(db_path)
        .with_context(|| format!("opening wallet db at {}", db_path.display()))?;

    let loaded = Wallet::load()
        .descriptor(
            KeychainKind::External,
            Some(descriptors.external_private.clone()),
        )
        .descriptor(
            KeychainKind::Internal,
            Some(descriptors.internal_private.clone()),
        )
        .extract_keys()
        .check_network(network)
        .load_wallet(&mut db)?;

    let wallet = match loaded {
        Some(wallet) => wallet,
        None => Wallet::create(
            descriptors.external_private.clone(),
            descriptors.internal_private.clone(),
        )
        .network(network)
        .create_wallet(&mut db)?,
    };

    Ok((wallet, db))
}

/// Sync the wallet against bitcoind: replay any blocks since the wallet's last-seen tip, then
/// pull in the current mempool. Persists after each step.
pub fn sync(
    wallet: &mut PersistedWallet<Connection>,
    db: &mut Connection,
    rpc: Arc<Client>,
) -> Result<()> {
    let wallet_tip = wallet.latest_checkpoint();
    let mut emitter = Emitter::new(
        rpc,
        wallet_tip,
        0,
        wallet
            .transactions()
            .filter(|tx| tx.chain_position.is_unconfirmed()),
    );

    let mut blocks = 0usize;
    while let Some(block_emission) = emitter.next_block()? {
        let height = block_emission.block_height();
        let connected_to = block_emission.connected_to();
        wallet.apply_block_connected_to(&block_emission.block, height, connected_to)?;
        wallet.persist(db)?;
        blocks += 1;
    }

    let mempool = emitter.mempool()?;
    wallet.apply_evicted_txs(mempool.evicted);
    wallet.apply_unconfirmed_txs(mempool.update);
    wallet.persist(db)?;

    println!(
        "Synced {blocks} new block(s); wallet tip is now {}.",
        wallet.latest_checkpoint().height()
    );
    Ok(())
}
