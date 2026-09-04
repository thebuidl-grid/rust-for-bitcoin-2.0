use bdk_bitcoind_rpc::bitcoincore_rpc::Client;
use bdk_bitcoind_rpc::Emitter;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, PersistedWallet, Wallet};

use crate::config::Config;

/// Open the wallet database, loading an existing wallet or creating one the first time
/// `init` is run. `Wallet::load` is what makes state survive a restart — the descriptors
/// are only needed again here to verify they match what's already on disk.
pub fn load_or_create(
    config: &Config,
    external_descriptor: String,
    internal_descriptor: String,
) -> anyhow::Result<(PersistedWallet<Connection>, Connection)> {
    let mut db = Connection::open(&config.db_path)?;

    let existing = Wallet::load()
        .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
        .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
        .extract_keys()
        .check_network(config.network)
        .load_wallet(&mut db)?;

    let wallet = match existing {
        Some(wallet) => wallet,
        None => Wallet::create(external_descriptor, internal_descriptor)
            .network(config.network)
            .create_wallet(&mut db)?,
    };

    Ok((wallet, db))
}

/// Pull new blocks and mempool state from bitcoind and fold them into the wallet.
///
/// This is what makes the wallet "not purely offline" — every command that needs a
/// current balance or UTXO set calls this first instead of trusting whatever's on disk.
pub fn sync(
    wallet: &mut PersistedWallet<Connection>,
    db: &mut Connection,
    client: &Client,
) -> anyhow::Result<()> {
    let wallet_tip = wallet.latest_checkpoint();
    let unconfirmed = wallet
        .transactions()
        .filter(|tx| tx.chain_position.is_unconfirmed())
        .map(|tx| tx.tx_node.tx.clone());

    let mut emitter = Emitter::new(client, wallet_tip, 0, unconfirmed);
    while let Some(block_emission) = emitter.next_block()? {
        let height = block_emission.block_height();
        let connected_to = block_emission.connected_to();
        wallet.apply_block_connected_to(&block_emission.block, height, connected_to)?;
    }

    let mempool_event = emitter.mempool()?;
    wallet.apply_evicted_txs(mempool_event.evicted);
    wallet.apply_unconfirmed_txs(mempool_event.update);

    wallet.persist(db)?;
    Ok(())
}
