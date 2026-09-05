//! One function per CLI subcommand. Each opens the local SQLite-backed
//! wallet (creating it on first run) and does its one job.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use bdk_bitcoind_rpc::bitcoincore_rpc::RpcApi;
use bdk_wallet::bitcoin::{Address, Amount};
use bdk_wallet::{KeychainKind, SignOptions};

use crate::config::Config;
use crate::descriptors::{load_or_create_wallet, open_db};
use crate::mnemonic;
use crate::sync::{rpc_client, sync_wallet};

/// `init`: generate (or reuse) the wallet's mnemonic, then create its local
/// descriptor wallet and SQLite store.
pub fn init(config: &mut Config, env_path: &std::path::Path) -> Result<()> {
    if config.mnemonic.is_none() {
        let phrase = mnemonic::generate()?;
        mnemonic::append_to_env_file(env_path, &phrase)?;
        println!(
            "Generated a new mnemonic and saved it to {}.",
            env_path.display()
        );
        println!(
            "Keep that file out of version control — it is the wallet's private key material."
        );
        config.mnemonic = Some(phrase);
    } else {
        println!("Using the mnemonic already configured in the environment.");
    }

    let mut db = open_db(config)?;
    let mut wallet = load_or_create_wallet(config, &mut db)?;
    let address = wallet.reveal_next_address(KeychainKind::External);
    wallet.persist(&mut db)?;

    println!("Network:            {}", config.network);
    println!("Descriptor type:    {:?}", config.descriptor_kind);
    println!("Account:            {}", config.account);
    println!("Wallet database:    {}", config.db_path.display());
    println!("First receive address: {}", address.address);
    Ok(())
}

/// `address`: reveal and print the next unused external address.
pub fn address(config: &Config) -> Result<()> {
    let mut db = open_db(config)?;
    let mut wallet = load_or_create_wallet(config, &mut db)?;
    let address = wallet.reveal_next_address(KeychainKind::External);
    wallet.persist(&mut db)?;
    println!("{}", address.address);
    Ok(())
}

/// `change-address`: reveal and print the next unused internal address.
pub fn change_address(config: &Config) -> Result<()> {
    let mut db = open_db(config)?;
    let mut wallet = load_or_create_wallet(config, &mut db)?;
    let address = wallet.reveal_next_address(KeychainKind::Internal);
    wallet.persist(&mut db)?;
    println!("{}", address.address);
    Ok(())
}

/// `sync`: sync local state against the configured node and report progress.
pub fn sync(config: &Config) -> Result<()> {
    let mut db = open_db(config)?;
    let mut wallet = load_or_create_wallet(config, &mut db)?;
    let client = rpc_client(config)?;
    let summary = sync_wallet(&mut wallet, &mut db, &client)?;
    println!(
        "Applied {} new block(s); wallet tip is now at height {}.",
        summary.blocks_applied, summary.tip_height
    );
    println!("Balance after sync: {}", wallet.balance().total());
    Ok(())
}

/// `balance`: sync, then print the full balance breakdown.
pub fn balance(config: &Config) -> Result<()> {
    let mut db = open_db(config)?;
    let mut wallet = load_or_create_wallet(config, &mut db)?;
    let client = rpc_client(config)?;
    sync_wallet(&mut wallet, &mut db, &client)?;

    let balance = wallet.balance();
    println!("Confirmed:          {}", balance.confirmed);
    println!("Trusted pending:    {}", balance.trusted_pending);
    println!("Untrusted pending:  {}", balance.untrusted_pending);
    println!("Immature (coinbase):{}", balance.immature);
    println!("Total:              {}", balance.total());
    Ok(())
}

/// `utxos`: sync, then list every UTXO the wallet knows about.
pub fn utxos(config: &Config) -> Result<()> {
    let mut db = open_db(config)?;
    let mut wallet = load_or_create_wallet(config, &mut db)?;
    let client = rpc_client(config)?;
    sync_wallet(&mut wallet, &mut db, &client)?;

    let mut count = 0usize;
    for utxo in wallet.list_unspent() {
        count += 1;
        println!(
            "{}:{}  {:>14}  {:?} #{}  {:?}",
            utxo.outpoint.txid,
            utxo.outpoint.vout,
            utxo.txout.value,
            utxo.keychain,
            utxo.derivation_index,
            utxo.chain_position,
        );
    }
    println!("{count} UTXO(s) total.");
    Ok(())
}

/// `send`: sync, build a transaction paying `amount_sats` to `address_str`,
/// sign it, broadcast it via RPC, and record it locally so a follow-up
/// `balance`/`utxos` reflects it immediately.
pub fn send(config: &Config, address_str: &str, amount_sats: u64) -> Result<()> {
    let mut db = open_db(config)?;
    let mut wallet = load_or_create_wallet(config, &mut db)?;
    let client = rpc_client(config)?;
    sync_wallet(&mut wallet, &mut db, &client)?;

    let address = address_str
        .parse::<Address<_>>()
        .with_context(|| format!("{address_str} is not a valid Bitcoin address"))?
        .require_network(config.network)
        .with_context(|| format!("{address_str} is not a {} address", config.network))?;

    let mut builder = wallet.build_tx();
    builder.add_recipient(address.script_pubkey(), Amount::from_sat(amount_sats));
    let mut psbt = builder
        .finish()
        .context("failed to build the transaction")?;

    let finalized = wallet
        .sign(&mut psbt, SignOptions::default())
        .context("failed to sign the transaction")?;
    anyhow::ensure!(
        finalized,
        "the wallet could not fully sign this transaction"
    );

    let tx = psbt
        .extract_tx()
        .context("failed to extract the final transaction")?;
    let txid = client
        .send_raw_transaction(&tx)
        .context("Bitcoin Core rejected the transaction")?;

    // Record it locally right away so this session's own balance/utxo views
    // reflect the spend without waiting for the next sync's mempool poll.
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    wallet.apply_unconfirmed_txs([(tx, now)]);
    wallet.persist(&mut db)?;

    println!("Broadcast transaction {txid}");
    println!("Paid {amount_sats} sats to {address}");
    Ok(())
}
