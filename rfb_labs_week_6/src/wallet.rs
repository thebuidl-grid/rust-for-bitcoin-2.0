//! Wallet lifecycle: create/load with SQLite persistence, and sync against a node.
//!
//! Key derivation and address generation go through `bdk_wallet` (built on `rust-bitcoin` +
//! `miniscript`), which owns the descriptor/keychain/UTXO-tracking logic. Chain sync goes through
//! `bdk_bitcoind_rpc`, which walks blocks and mempool from a `bitcoincore-rpc` client and turns
//! them into wallet updates.

use anyhow::{Context, Result};
use bdk_bitcoind_rpc::Emitter;
use bdk_wallet::bitcoin::NetworkKind;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::descriptor::template::Bip84;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, PersistedWallet, Wallet};
use bitcoincore_rpc::Client;

use crate::config::Config;

/// The wallet type used throughout this app: a BDK `Wallet` persisted to a local SQLite file.
pub type AppWallet = PersistedWallet<Connection>;

/// Derive the BIP32 master key this wallet is rooted at, from the configured mnemonic.
///
/// BIP84 (native SegWit / `wpkh`) is used for both keychains -- see the README for why.
pub fn master_key(config: &Config) -> Xpriv {
    let seed = config.mnemonic.to_seed(&config.passphrase);
    let network_kind: NetworkKind = config.network.into();
    Xpriv::new_master(network_kind, &seed)
        .expect("a 64-byte BIP39 seed always yields a valid xpriv")
}

/// Load the wallet from `db_path` if it already exists there, otherwise create it fresh.
///
/// The same BIP84 external/internal descriptor templates are supplied on both paths: on `load`
/// they are used only to restore the private signing keys (`.extract_keys()`) and confirmed
/// against what's already on disk; on `create` they define the wallet from scratch.
pub fn open(config: &Config, db: &mut Connection) -> Result<AppWallet> {
    let xpriv = master_key(config);
    let external = Bip84(xpriv, KeychainKind::External);
    let internal = Bip84(xpriv, KeychainKind::Internal);

    let loaded = Wallet::load()
        .descriptor(KeychainKind::External, Some(external))
        .descriptor(KeychainKind::Internal, Some(internal))
        .extract_keys()
        .check_network(config.network)
        .load_wallet(db)
        .context("failed to load an existing wallet from the database")?;

    match loaded {
        Some(wallet) => Ok(wallet),
        None => {
            let external = Bip84(xpriv, KeychainKind::External);
            let internal = Bip84(xpriv, KeychainKind::Internal);
            Wallet::create(external, internal)
                .network(config.network)
                .create_wallet(db)
                .context("failed to create a new wallet")
        }
    }
}

/// Walk the node's chain and mempool from where the wallet last left off, applying every
/// relevant block/transaction to the wallet's state and persisting after each step.
pub fn sync(wallet: &mut AppWallet, db: &mut Connection, client: &Client) -> Result<()> {
    let wallet_tip = wallet.latest_checkpoint();
    let start_height = 0;

    let mut emitter = Emitter::new(
        client,
        wallet_tip,
        start_height,
        wallet
            .transactions()
            .filter(|tx| tx.chain_position.is_unconfirmed()),
    );

    let mut blocks_applied = 0_u32;
    while let Some(block_emission) = emitter.next_block().context("failed to fetch next block")? {
        let height = block_emission.block_height();
        let connected_to = block_emission.connected_to();
        wallet.apply_block_connected_to(&block_emission.block, height, connected_to)?;
        wallet
            .persist(db)
            .context("failed to persist wallet after applying a block")?;
        blocks_applied += 1;
    }

    let mempool_event = emitter.mempool().context("failed to fetch the mempool")?;
    wallet.apply_evicted_txs(mempool_event.evicted);
    wallet.apply_unconfirmed_txs(mempool_event.update);
    wallet
        .persist(db)
        .context("failed to persist wallet after applying mempool state")?;

    println!("Synced: applied {blocks_applied} new block(s).");
    Ok(())
}
