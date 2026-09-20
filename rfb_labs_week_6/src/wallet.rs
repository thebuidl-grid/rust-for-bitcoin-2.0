use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use bdk_bitcoind_rpc::Emitter;
use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client};
use bdk_wallet::bitcoin::Network;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, PersistedWallet, Wallet};
use bip39::Mnemonic;

use crate::config::Config;

const WALLET_DB: &str = "wallet.sqlite";

fn build_descriptors(xpriv: Xpriv, network: Network) -> (String, String) {
    let coin_type = if network == Network::Bitcoin { 0 } else { 1 };
    let external = format!("wpkh({xpriv}/84h/{coin_type}h/0h/0/*)");
    let internal = format!("wpkh({xpriv}/84h/{coin_type}h/0h/1/*)");
    (external, internal)
}

/// Open (or create) the wallet and its SQLite connection together, since
/// syncing needs to persist to the same connection repeatedly afterward.
pub fn open_or_create(
    mnemonic: &Mnemonic,
    network: Network,
) -> Result<(PersistedWallet<Connection>, Connection)> {
    let seed = mnemonic.to_seed("");
    let xpriv = Xpriv::new_master(network, &seed).context("failed to derive master key")?;
    let (external_descriptor, internal_descriptor) = build_descriptors(xpriv, network);

    let mut conn = Connection::open(Path::new(WALLET_DB))
        .with_context(|| format!("failed to open wallet database at {WALLET_DB}"))?;

    let wallet_opt = Wallet::load()
        .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
        .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
        .extract_keys()
        .check_network(network)
        .load_wallet(&mut conn)
        .context("failed to load existing wallet from database")?;

    let wallet = match wallet_opt {
        Some(wallet) => {
            println!("Loaded existing wallet from {WALLET_DB}.");
            wallet
        }
        None => {
            println!("No existing wallet found — creating a new one at {WALLET_DB}.");
            Wallet::create(external_descriptor, internal_descriptor)
                .network(network)
                .create_wallet(&mut conn)
                .context("failed to create new wallet")?
        }
    };

    Ok((wallet, conn))
}

/// Sync the wallet against a Bitcoin Core node: walk forward from the
/// wallet's last known checkpoint, applying every new block since, then
/// pick up any relevant unconfirmed mempool transactions. This is what
/// turns "a wallet that has addresses" into "a wallet that knows its own
/// UTXOs and balance."
pub fn sync(
    wallet: &mut PersistedWallet<Connection>,
    db: &mut Connection,
    config: &Config,
) -> Result<()> {
    let rpc_client = Arc::new(
        Client::new(
            &config.rpc_url,
            Auth::UserPass(config.rpc_user.clone(), config.rpc_password.clone()),
        )
        .context("failed to build RPC client for syncing")?,
    );

    let wallet_tip = wallet.latest_checkpoint();
    let mut emitter = Emitter::new(
        rpc_client,
        wallet_tip,
        0,
        wallet
            .transactions()
            .filter(|tx| tx.chain_position.is_unconfirmed()),
    );

    let mut blocks_applied = 0_usize;
    while let Some(block_emission) = emitter
        .next_block()
        .context("failed to fetch next block during sync")?
    {
        let height = block_emission.block_height();
        let connected_to = block_emission.connected_to();
        wallet
            .apply_block_connected_to(&block_emission.block, height, connected_to)
            .context("failed to apply block to wallet")?;
        wallet
            .persist(db)
            .context("failed to persist wallet after block")?;
        blocks_applied += 1;
    }

    let mempool_event = emitter.mempool().context("failed to fetch mempool")?;
    wallet.apply_evicted_txs(mempool_event.evicted);
    wallet.apply_unconfirmed_txs(mempool_event.update);
    wallet
        .persist(db)
        .context("failed to persist wallet after mempool sync")?;

    println!("Sync complete: applied {blocks_applied} block(s).");

    Ok(())
}
