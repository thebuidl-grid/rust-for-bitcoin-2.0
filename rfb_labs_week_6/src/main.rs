mod chain;
mod config;
mod keys;
mod wallet_store;

use std::str::FromStr;

use anyhow::{Context, Result, bail};
use bdk_bitcoind_rpc::bitcoincore_rpc::{Client, RpcApi};
use bdk_wallet::bitcoin::{Address, Amount, FeeRate, Network};
use bdk_wallet::{KeychainKind, SignOptions};
use clap::Parser;

use config::{Cli, Command, Config};

fn main() -> Result<()> {
    // Load `.env` (if present) before clap reads environment variables, so a wallet created by
    // `init` is picked up automatically by every later command run from the same directory.
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    let cfg = cli.config;

    match cli.command {
        Command::Init { kind, force } => cmd_init(&cfg, kind, force),
        Command::Address => cmd_address(&cfg, KeychainKind::External),
        Command::ChangeAddress => cmd_address(&cfg, KeychainKind::Internal),
        Command::Sync => cmd_sync(&cfg),
        Command::Balance => cmd_balance(&cfg),
        Command::Utxos => cmd_utxos(&cfg),
        Command::Send {
            to,
            amount,
            fee_rate,
        } => cmd_send(&cfg, &to, amount, fee_rate),
        Command::Mine { blocks } => cmd_mine(&cfg, blocks),
    }
}

fn cmd_init(cfg: &Config, kind: config::DescriptorKind, force: bool) -> Result<()> {
    let (mut wallet, mut db, generated) = wallet_store::init(cfg, kind, force)?;
    let address = wallet.reveal_next_address(KeychainKind::External).address;
    wallet
        .persist(&mut db)
        .context("failed to persist wallet state")?;

    println!("Wallet initialized on {}.", cfg.network);
    println!();
    println!("Recovery phrase (write this down, it is only shown once):");
    println!("  {}", generated.mnemonic);
    println!();
    println!("First receiving address: {address}");
    println!("Descriptors and the phrase above were written to .env — do not commit that file.");
    Ok(())
}

fn cmd_address(cfg: &Config, keychain: KeychainKind) -> Result<()> {
    let (mut wallet, mut db) = wallet_store::open(cfg)?;
    let info = wallet.reveal_next_address(keychain);
    wallet
        .persist(&mut db)
        .context("failed to persist wallet state")?;
    println!("{}", info.address);
    Ok(())
}

fn cmd_sync(cfg: &Config) -> Result<()> {
    let (mut wallet, mut db) = wallet_store::open(cfg)?;
    let client = chain::connect(cfg)?;
    let blocks = chain::sync_wallet(&mut wallet, &mut db, &client, cfg.start_height)?;
    println!("Synced {blocks} new block(s).");
    println!(
        "Tip: {} at height {}",
        wallet.latest_checkpoint().hash(),
        wallet.latest_checkpoint().height()
    );
    println!("Balance: {}", wallet.balance().total());
    Ok(())
}

fn cmd_balance(cfg: &Config) -> Result<()> {
    let (mut wallet, mut db) = wallet_store::open(cfg)?;
    let client = chain::connect(cfg)?;
    chain::sync_wallet(&mut wallet, &mut db, &client, cfg.start_height)?;

    let balance = wallet.balance();
    println!("confirmed:         {}", balance.confirmed);
    println!("trusted pending:   {}", balance.trusted_pending);
    println!("untrusted pending: {}", balance.untrusted_pending);
    println!("immature:          {}", balance.immature);
    println!("total:             {}", balance.total());
    Ok(())
}

fn cmd_utxos(cfg: &Config) -> Result<()> {
    let (mut wallet, mut db) = wallet_store::open(cfg)?;
    let client = chain::connect(cfg)?;
    chain::sync_wallet(&mut wallet, &mut db, &client, cfg.start_height)?;

    let mut count = 0;
    for utxo in wallet.list_unspent() {
        count += 1;
        println!(
            "{}:{}  {: >14}  {:?}  derivation index {}",
            utxo.outpoint.txid,
            utxo.outpoint.vout,
            utxo.txout.value,
            utxo.keychain,
            utxo.derivation_index
        );
    }
    if count == 0 {
        println!(
            "No UTXOs. Fund the wallet with `send` from another wallet, or `mine` on regtest."
        );
    }
    Ok(())
}

fn cmd_send(cfg: &Config, to: &str, amount_sats: u64, fee_rate_sat_vb: u64) -> Result<()> {
    let (mut wallet, mut db) = wallet_store::open(cfg)?;
    let client = chain::connect(cfg)?;
    chain::sync_wallet(&mut wallet, &mut db, &client, cfg.start_height)?;

    let to_address = Address::from_str(to)
        .with_context(|| format!("'{to}' is not a valid Bitcoin address"))?
        .require_network(cfg.network)
        .with_context(|| format!("'{to}' is not a valid address for {}", cfg.network))?;
    let fee_rate =
        FeeRate::from_sat_per_vb(fee_rate_sat_vb).context("fee rate is too large to represent")?;

    let mut psbt = {
        let mut builder = wallet.build_tx();
        builder.add_recipient(to_address.script_pubkey(), Amount::from_sat(amount_sats));
        builder.fee_rate(fee_rate);
        builder
            .finish()
            .context("failed to build transaction (insufficient funds?)")?
    };

    // `TxBuilder` only attaches `witness_utxo` (not the full `non_witness_utxo` previous
    // transaction) to each PSBT input. `SignOptions::default()` refuses to sign from
    // `witness_utxo` alone as a mitigation for the "SegWit bug" (a malicious PSBT lying about an
    // input's value to under-report the fee). That protects against *foreign* PSBTs; here the
    // PSBT was built by this same wallet from its own synced chain data, so trusting it is safe.
    let sign_options = SignOptions {
        trust_witness_utxo: true,
        ..Default::default()
    };
    let finalized = wallet
        .sign(&mut psbt, sign_options)
        .context("failed to sign transaction")?;
    if !finalized {
        bail!("wallet could not fully sign the transaction");
    }

    let tx = psbt
        .extract_tx()
        .context("failed to extract finalized transaction")?;
    let txid = client
        .send_raw_transaction(&tx)
        .context("bitcoind rejected the transaction")?;
    wallet
        .persist(&mut db)
        .context("failed to persist wallet state")?;

    println!("Broadcast txid: {txid}");
    println!(
        "New balance (unconfirmed until mined): {}",
        wallet.balance().total()
    );
    Ok(())
}

/// Regtest-only helper: mines blocks directly to the wallet's own next address, so `init` +
/// `mine` is enough to get a funded wallet with no separate bitcoind wallet required (the
/// `Emitter`/`bitcoincore-rpc` sync path here works against wallet-disabled nodes too).
fn cmd_mine(cfg: &Config, blocks: u32) -> Result<()> {
    if cfg.network != Network::Regtest {
        bail!(
            "`mine` only makes sense on regtest (network is currently {})",
            cfg.network
        );
    }
    let (mut wallet, mut db) = wallet_store::open(cfg)?;
    let client = chain::connect(cfg)?;

    let address = wallet.reveal_next_address(KeychainKind::External).address;
    wallet
        .persist(&mut db)
        .context("failed to persist wallet state")?;

    let hashes = mine_to(&client, blocks, &address)?;
    println!("Mined {} block(s) to {address}.", hashes.len());

    chain::sync_wallet(&mut wallet, &mut db, &client, cfg.start_height)?;
    println!("Balance after sync: {}", wallet.balance().total());
    Ok(())
}

fn mine_to(
    client: &Client,
    blocks: u32,
    address: &Address,
) -> Result<Vec<bdk_wallet::bitcoin::BlockHash>> {
    client
        .generate_to_address(blocks as u64, address)
        .context("generatetoaddress failed")
}
