//! Wallet logic: creating/loading the wallet, syncing it from Bitcoin Core, and building/signing
//! transactions. Everything here goes through `bdk_wallet` (descriptors, keychains, UTXOs,
//! signing) and `bdk_bitcoind_rpc` (turning a Bitcoin Core node into a chain source for BDK) --
//! this module is deliberately thin glue around those two crates rather than reimplementing any
//! of what they already do.

use std::str::FromStr;
use std::sync::Arc;

use bdk_bitcoind_rpc::Emitter;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::{Address, Amount, FeeRate, Network};
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::keys::GeneratableKey;
use bdk_wallet::miniscript::Segwitv0;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::template::Bip84;
use bdk_wallet::{Balance, KeychainKind, PersistedWallet, SignOptions, Wallet};
use bitcoincore_rpc::{Client, RpcApi};

use crate::db;
use crate::error::{Result, WalletError};

/// Fixed feerate used for every transaction we build. Bitcoin Core's regtest node has no mempool
/// history to estimate fees from, so dynamic fee estimation is out of scope here -- see the
/// README's "Limitations" section.
const FEE_RATE_SAT_VB: u64 = 2;

/// Where the emitter starts scanning from. `0` is fine for a regtest demo chain, which is only
/// ever a few hundred blocks deep at most.
const SYNC_START_HEIGHT: u32 = 0;

pub type WalletHandle = PersistedWallet<Connection>;

/// Derives the wallet's master extended private key from a BIP39 mnemonic phrase.
///
/// This is the one place we use `rust-bitcoin`'s BIP32 type directly instead of going through a
/// BDK helper: `bdk_wallet::bitcoin::bip32::Xpriv::new_master` turns a raw seed into the root key
/// that everything else (the two BIP84 descriptors below) is derived from. See the README for why
/// this particular spot calls `rust-bitcoin` directly.
fn xprv_from_mnemonic(mnemonic: &Mnemonic, network: Network) -> Result<Xpriv> {
    let seed = mnemonic.to_seed("");
    Xpriv::new_master(network, &seed).map_err(WalletError::app)
}

/// Loads the wallet's mnemonic from the database, generating and persisting a fresh one the very
/// first time the wallet is opened. This mnemonic is the wallet's only secret; everything else
/// (address indices, UTXOs, transaction history) lives in BDK's own SQLite tables and is
/// re-derived / re-synced from the chain, never regenerated.
fn load_or_create_mnemonic(conn: &Connection) -> Result<Mnemonic> {
    if let Some(phrase) = db::load_mnemonic(conn)? {
        return Mnemonic::parse_in(Language::English, &phrase).map_err(WalletError::app);
    }
    let generated: bdk_wallet::keys::GeneratedKey<Mnemonic, Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|e| WalletError::App(format!("mnemonic generation failed: {e:?}")))?;
    let mnemonic: Mnemonic = generated.into_key();
    db::save_mnemonic(conn, &mnemonic.to_string())?;
    Ok(mnemonic)
}

/// Opens the wallet, creating it (and a fresh seed) on first run.
///
/// Uses a BIP84 (`wpkh`) descriptor for both keychains:
/// - external/receiving: `wpkh([fp/84'/1'/0']tprv.../0/*)`
/// - internal/change:    `wpkh([fp/84'/1'/0']tprv.../1/*)`
///
/// The private descriptors (with the signing keys embedded) are re-derived from the mnemonic on
/// every call -- they are never written to the database -- and handed to BDK so it can extract
/// signers via `.extract_keys()`. BDK persists only the *public* form of these descriptors plus
/// all chain state (addresses revealed, transactions, UTXOs) in its own SQLite tables.
pub fn open_wallet(conn: &mut Connection, network: Network) -> Result<WalletHandle> {
    let mnemonic = load_or_create_mnemonic(conn)?;
    let xprv = xprv_from_mnemonic(&mnemonic, network)?;

    let external = Bip84(xprv, KeychainKind::External);
    let internal = Bip84(xprv, KeychainKind::Internal);

    let loaded = Wallet::load()
        .descriptor(KeychainKind::External, Some(external.clone()))
        .descriptor(KeychainKind::Internal, Some(internal.clone()))
        .extract_keys()
        .check_network(network)
        .load_wallet(conn)
        .map_err(WalletError::app)?;

    match loaded {
        Some(wallet) => Ok(wallet),
        None => Wallet::create(external, internal)
            .network(network)
            .create_wallet(conn)
            .map_err(WalletError::app),
    }
}

/// Connects to Bitcoin Core, walks the chain block-by-block from the wallet's last known tip, and
/// applies mempool state, updating balances/UTXOs. This is the BDK-supported way to sync a wallet
/// against a raw Bitcoin Core node (no Electrum/Esplora indexer required): `bdk_bitcoind_rpc`'s
/// `Emitter` reads blocks and mempool contents straight from the node's RPC interface.
pub fn sync(wallet: &mut WalletHandle, conn: &mut Connection, rpc: Arc<Client>) -> Result<u32> {
    let wallet_tip = wallet.latest_checkpoint();
    let unconfirmed = wallet
        .transactions()
        .filter(|tx| tx.chain_position.is_unconfirmed())
        .map(|tx| tx.tx_node.tx.clone());

    let mut emitter = Emitter::new(rpc, wallet_tip, SYNC_START_HEIGHT, unconfirmed);

    let mut blocks_applied = 0u32;
    while let Some(block_emission) = emitter.next_block().map_err(WalletError::from)? {
        let height = block_emission.block_height();
        let connected_to = block_emission.connected_to();
        wallet
            .apply_block_connected_to(&block_emission.block, height, connected_to)
            .map_err(WalletError::app)?;
        blocks_applied += 1;
    }

    let mempool_event = emitter.mempool().map_err(WalletError::from)?;
    wallet.apply_evicted_txs(mempool_event.evicted);
    wallet.apply_unconfirmed_txs(mempool_event.update);

    wallet.persist(conn).map_err(WalletError::from)?;
    Ok(blocks_applied)
}

/// Reveals (and persists) the next unused address for the given keychain.
pub fn new_address(
    wallet: &mut WalletHandle,
    conn: &mut Connection,
    keychain: KeychainKind,
) -> Result<Address> {
    let info = wallet.reveal_next_address(keychain);
    wallet.persist(conn).map_err(WalletError::from)?;
    Ok(info.address)
}

pub fn balance(wallet: &WalletHandle) -> Balance {
    wallet.balance()
}

/// Builds, signs and broadcasts a transaction sending `amount` to `to`. Change (if any) is sent
/// to the internal/change keychain automatically by `TxBuilder`'s default coin selection.
///
/// Returns the broadcast transaction's txid.
pub fn send(
    wallet: &mut WalletHandle,
    conn: &mut Connection,
    rpc: &Client,
    to: &str,
    amount_btc: f64,
) -> Result<bdk_wallet::bitcoin::Txid> {
    let network = wallet.network();
    let address = Address::from_str(to)
        .map_err(WalletError::app)?
        .require_network(network)
        .map_err(WalletError::app)?;
    let amount = Amount::from_btc(amount_btc).map_err(WalletError::app)?;
    let fee_rate = FeeRate::from_sat_per_vb(FEE_RATE_SAT_VB)
        .ok_or_else(|| WalletError::App("invalid fee rate".into()))?;

    let mut builder = wallet.build_tx();
    builder.add_recipient(address.script_pubkey(), amount);
    builder.fee_rate(fee_rate);
    let mut psbt = builder.finish().map_err(WalletError::app)?;
    println!("Transaction created.");

    let finalized = wallet
        .sign(&mut psbt, SignOptions::default())
        .map_err(WalletError::app)?;
    if !finalized {
        return Err(WalletError::App(
            "wallet could not fully sign the transaction".into(),
        ));
    }
    println!("Transaction signed.");

    let tx = psbt.extract_tx().map_err(WalletError::app)?;
    rpc.send_raw_transaction(&tx)?;
    println!("Transaction broadcast.");

    // Make the wallet aware of its own just-broadcast transaction immediately, rather than
    // waiting for the next explicit `sync` (which would also pick it up from the mempool).
    let seen_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    wallet.apply_unconfirmed_txs([(tx.clone(), seen_at)]);
    wallet.persist(conn).map_err(WalletError::from)?;

    Ok(tx.compute_txid())
}
