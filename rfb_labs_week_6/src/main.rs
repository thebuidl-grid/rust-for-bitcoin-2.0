use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use bdk_bitcoind_rpc::{Emitter, NO_EXPECTED_MEMPOOL_TXS};
use bdk_sqlite::Store;
use bdk_wallet::{KeychainKind, LoadParams, PersistedWallet, SignOptions, Wallet, bitcoin};
use bitcoin::{
    Address, Amount, Network,
    bip32::{ChildNumber, DerivationPath, Xpriv},
};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use clap::{Parser, Subcommand, ValueEnum};
use rand::{RngCore, rngs::OsRng};

const NETWORK: Network = Network::Regtest;
const DATABASE: &str = "wallet.sqlite";

#[derive(Debug, Parser)]
#[command(author, version, about = "A small BDK Bitcoin Core regtest wallet")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    Address {
        #[arg(value_enum, default_value_t = Keychain::External)]
        keychain: Keychain,
    },
    Sync,
    Balance,
    Utxos,
    Send {
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount_sats: u64,
        #[arg(long, default_value_t = 2)]
        fee_rate: u64,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Keychain {
    External,
    Internal,
}

impl From<Keychain> for KeychainKind {
    fn from(value: Keychain) -> Self {
        match value {
            Keychain::External => Self::External,
            Keychain::Internal => Self::Internal,
        }
    }
}

#[derive(Clone, Debug)]
struct Config {
    mnemonic: String,
    rpc_url: String,
    rpc_user: String,
    rpc_password: String,
}

fn rpc_client(config: &Config) -> Result<Client> {
    let cookie = dotenv_value("BITCOIN_RPC_COOKIE")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("LOCALAPPDATA").map(|appdata| {
                PathBuf::from(appdata)
                    .join("Bitcoin")
                    .join("regtest")
                    .join(".cookie")
            })
        })
        .or_else(|| {
            env::var_os("APPDATA").map(|appdata| {
                PathBuf::from(appdata)
                    .join("Bitcoin")
                    .join("regtest")
                    .join(".cookie")
            })
        });
    let auth = cookie
        .filter(|path| path.is_file())
        .map(Auth::CookieFile)
        .unwrap_or_else(|| Auth::UserPass(config.rpc_user.clone(), config.rpc_password.clone()));
    Client::new(&config.rpc_url, auth).context("connecting to Bitcoin Core RPC")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(run(cli.command))
}

async fn run(command: Command) -> Result<()> {
    if matches!(command, Command::Init) {
        init().await
    } else {
        let config = load_config()?;
        let mut db = Store::new(DATABASE)
            .await
            .context("opening SQLite wallet store")?;
        let (external, internal) = descriptors(&config.mnemonic)?;
        let params = LoadParams::new()
            .descriptor(KeychainKind::External, Some(external))
            .descriptor(KeychainKind::Internal, Some(internal))
            .extract_keys()
            .check_network(NETWORK);
        let mut wallet = PersistedWallet::load_async(&mut db, params)
            .await
            .context("loading wallet state")?
            .context("wallet is not initialized; run `cargo run -- init` first")?;

        match command {
            Command::Address { keychain } => address(&mut wallet, &mut db, keychain.into()).await,
            Command::Sync => sync(&mut wallet, &mut db, &config).await,
            Command::Balance => balance(&wallet),
            Command::Utxos => utxos(&wallet),
            Command::Send {
                to,
                amount_sats,
                fee_rate,
            } => send(&mut wallet, &mut db, &config, &to, amount_sats, fee_rate).await,
            Command::Init => unreachable!(),
        }
    }
}

async fn init() -> Result<()> {
    if Path::new(DATABASE).exists() {
        bail!("{DATABASE} already exists; refusing to overwrite wallet state")
    }
    let mnemonic = match dotenv_value("WALLET_MNEMONIC") {
        Some(value) => {
            bdk_wallet::keys::bip39::Mnemonic::parse(&value)
                .context("WALLET_MNEMONIC is not a valid BIP39 mnemonic")?;
            value
        }
        None => {
            let mut entropy = [0_u8; 16];
            OsRng.fill_bytes(&mut entropy);
            let generated = bdk_wallet::keys::bip39::Mnemonic::from_entropy_in(
                bdk_wallet::keys::bip39::Language::English,
                &entropy,
            )
            .map_err(|error| anyhow!("generating mnemonic: {error:?}"))?;
            let value = generated.to_string();
            write_env(&value)?;
            println!("Generated mnemonic and wrote it to .env. Back it up securely.");
            value
        }
    };
    let (external, internal) = descriptors(&mnemonic)?;
    let mut db = Store::new(DATABASE)
        .await
        .context("opening SQLite wallet store")?;
    let mut wallet = Wallet::create(external, internal)
        .network(NETWORK)
        .create_wallet_async(&mut db)
        .await
        .context("creating wallet")?;
    wallet
        .persist_async(&mut db)
        .await
        .context("persisting wallet")?;
    println!("Initialized regtest Taproot wallet in {DATABASE}");
    println!(
        "External descriptor: {}",
        wallet.public_descriptor(KeychainKind::External)
    );
    println!(
        "Internal descriptor: {}",
        wallet.public_descriptor(KeychainKind::Internal)
    );
    Ok(())
}

fn load_config() -> Result<Config> {
    Ok(Config {
        mnemonic: dotenv_value("WALLET_MNEMONIC")
            .context("WALLET_MNEMONIC is missing from the environment and .env; run init")?,
        rpc_url: dotenv_value("BITCOIN_RPC_URL").unwrap_or_else(|| "http://127.0.0.1:18443".into()),
        rpc_user: dotenv_value("BITCOIN_RPC_USER").unwrap_or_else(|| "bitcoin".into()),
        rpc_password: dotenv_value("BITCOIN_RPC_PASSWORD").unwrap_or_else(|| "bitcoin".into()),
    })
}

fn env_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env")
}

/// Read a non-empty setting from the process environment or either supported
/// `.env` location without printing secrets to diagnostics.
fn dotenv_value(name: &str) -> Option<String> {
    if let Ok(value) = env::var(name) {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }

    let paths = [PathBuf::from(".env"), env_path()];
    for path in paths {
        let Ok(entries) = dotenvy::from_path_iter(&path) else {
            if let Some(value) = read_env_line(&path, name) {
                return Some(value);
            }
            continue;
        };
        for entry in entries.flatten() {
            if entry.0 == name && !entry.1.trim().is_empty() {
                return Some(entry.1);
            }
        }
        if let Some(value) = read_env_line(&path, name) {
            return Some(value);
        }
    }
    None
}

fn read_env_line(path: &Path, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    fs::read_to_string(path)
        .ok()?
        .lines()
        .map(str::trim)
        .find_map(|line| {
            line.strip_prefix(&prefix)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
}

fn write_env(mnemonic: &str) -> Result<()> {
    let path = env_path();
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<&str> = existing
        .lines()
        .filter(|line| !line.trim_start().starts_with("WALLET_MNEMONIC="))
        .collect();
    lines.push("WALLET_MNEMONIC=");
    let mut output = lines.join("\n");
    output = output.replacen(
        "WALLET_MNEMONIC=",
        &format!("WALLET_MNEMONIC={mnemonic}"),
        1,
    );
    if !output.ends_with('\n') {
        output.push('\n');
    }
    fs::write(path, output)?;
    Ok(())
}

fn descriptors(words: &str) -> Result<(String, String)> {
    let mnemonic = bdk_wallet::keys::bip39::Mnemonic::parse(words).context("invalid mnemonic")?;
    let seed = mnemonic.to_seed("");
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let master = Xpriv::new_master(NETWORK, &seed)?;
    let account_path = DerivationPath::from(vec![
        ChildNumber::from_hardened_idx(86)?,
        ChildNumber::from_hardened_idx(1)?,
        ChildNumber::from_hardened_idx(0)?,
    ]);
    let account = master.derive_priv(&secp, &account_path)?;
    let origin = master.fingerprint(&secp);
    let external = format!("tr([{origin}/86'/1'/0']{account}/0/*)");
    let internal = format!("tr([{origin}/86'/1'/0']{account}/1/*)");
    Ok((external, internal))
}

async fn address(
    wallet: &mut PersistedWallet<Store>,
    db: &mut Store,
    keychain: KeychainKind,
) -> Result<()> {
    let info = wallet.reveal_next_address(keychain);
    wallet
        .persist_async(db)
        .await
        .context("persisting revealed address")?;
    println!(
        "keychain={keychain:?} index={} address={}",
        info.index, info.address
    );
    Ok(())
}

async fn sync(wallet: &mut PersistedWallet<Store>, db: &mut Store, config: &Config) -> Result<()> {
    let rpc = rpc_client(config)?;
    let checkpoint = wallet.latest_checkpoint();
    let mut emitter = Emitter::new(
        &rpc,
        checkpoint.clone(),
        checkpoint.height() + 1,
        NO_EXPECTED_MEMPOOL_TXS,
    );
    let mut blocks = 0;
    while let Some(event) = emitter
        .next_block()
        .context("reading block from Bitcoin Core")?
    {
        wallet
            .apply_block_connected_to(&event.block, event.block_height(), event.connected_to())
            .context("applying block to wallet")?;
        blocks += 1;
    }
    let mempool = emitter.mempool().context("reading Bitcoin Core mempool")?;
    wallet.apply_unconfirmed_txs(mempool.update);
    wallet.apply_evicted_txs(mempool.evicted);
    wallet
        .persist_async(db)
        .await
        .context("persisting synchronized wallet")?;
    println!(
        "Synced {blocks} block(s); balance available={} sats",
        wallet.balance().trusted_spendable().to_sat()
    );
    Ok(())
}

fn balance(wallet: &PersistedWallet<Store>) -> Result<()> {
    let b = wallet.balance();
    println!("available={} sats", b.trusted_spendable().to_sat());
    println!("trusted_pending={} sats", b.trusted_pending.to_sat());
    println!("untrusted_pending={} sats", b.untrusted_pending.to_sat());
    println!("immature={} sats", b.immature.to_sat());
    Ok(())
}

fn utxos(wallet: &PersistedWallet<Store>) -> Result<()> {
    for output in wallet.list_unspent() {
        let keychain = wallet
            .derivation_of_spk(output.txout.script_pubkey.clone())
            .map(|(k, _)| format!("{k:?}"))
            .unwrap_or_else(|| "unknown".into());
        println!(
            "{}:{} value={} sats keychain={keychain}",
            output.outpoint.txid,
            output.outpoint.vout,
            output.txout.value.to_sat()
        );
    }
    Ok(())
}

async fn send(
    wallet: &mut PersistedWallet<Store>,
    db: &mut Store,
    config: &Config,
    destination: &str,
    sats: u64,
    fee_rate: u64,
) -> Result<()> {
    if sats == 0 {
        bail!("amount_sats must be greater than zero")
    }
    let address = destination
        .parse::<Address<_>>()
        .context("invalid destination address")?
        .require_network(NETWORK)
        .context("destination is not a regtest address")?;
    let mut builder = wallet.build_tx();
    builder.add_recipient(address.script_pubkey(), Amount::from_sat(sats));
    builder.fee_rate(bitcoin::FeeRate::from_sat_per_vb(fee_rate).context("invalid fee rate")?);
    let mut psbt = builder
        .finish()
        .context("building transaction; sync and fund the wallet first")?;
    let finalized = wallet
        .sign(&mut psbt, SignOptions::default())
        .context("signing transaction")?;
    if !finalized {
        bail!("transaction could not be finalized")
    }
    let tx = psbt.extract_tx().context("extracting signed transaction")?;
    let rpc = rpc_client(config)?;
    let txid = rpc.send_raw_transaction(&bitcoin::consensus::serialize(&tx))?;
    wallet
        .persist_async(db)
        .await
        .context("persisting sent transaction")?;
    println!("broadcast txid={txid}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_have_separate_keychains() {
        let (external, internal) = descriptors("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap();
        assert_ne!(external, internal);
        assert!(external.contains("/0/*"));
        assert!(internal.contains("/1/*"));
    }

    #[test]
    fn destination_must_be_regtest() {
        let address = "tb1qexample".parse::<Address<_>>();
        assert!(address.is_err());
    }
}
