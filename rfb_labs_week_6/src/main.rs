use anyhow::{Context, Result, bail};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::key::CompressedPublicKey;
use bitcoin::{Address, Amount, Network};
use bitcoincore_rpc::{Auth, Client, RpcApi, json};
use clap::{Parser, Subcommand, ValueEnum};
use rand::rngs::OsRng;
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Parser)]
#[command(about = "A descriptor-based Bitcoin Core wallet")]
struct Cli {
    #[arg(long, default_value = "wallet.db", global = true)]
    database: PathBuf,
    #[arg(
        long,
        env = "BITCOIN_RPC_URL",
        default_value = "http://127.0.0.1:18443",
        global = true
    )]
    rpc_url: String,
    #[arg(long, env = "BITCOIN_RPC_USER", global = true)]
    rpc_user: Option<String>,
    #[arg(long, env = "BITCOIN_RPC_PASSWORD", global = true)]
    rpc_password: Option<String>,
    #[arg(long, value_enum, default_value_t = WalletNetwork::Regtest, global = true)]
    network: WalletNetwork,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum WalletNetwork {
    Regtest,
    Testnet,
}

impl WalletNetwork {
    fn bitcoin(self) -> Network {
        match self {
            Self::Regtest => Network::Regtest,
            Self::Testnet => Network::Testnet,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    Address {
        #[arg(value_enum)]
        keychain: Keychain,
    },
    Balance,
    Sync,
    Send {
        address: String,
        amount_btc: f64,
    },
    ShowDescriptor,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Keychain {
    External,
    Internal,
}

struct Wallet {
    db: Connection,
    network: Network,
}

impl Wallet {
    fn open(path: &PathBuf, network: Network) -> Result<Self> {
        let db = Connection::open(path).context("opening wallet database")?;
        db.execute_batch("PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS wallet (id INTEGER PRIMARY KEY CHECK (id = 1), xpriv TEXT NOT NULL, next_external INTEGER NOT NULL DEFAULT 0, next_internal INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS addresses (address TEXT PRIMARY KEY, keychain TEXT NOT NULL, child_index INTEGER NOT NULL, used INTEGER NOT NULL DEFAULT 0, UNIQUE(keychain, child_index));
            CREATE TABLE IF NOT EXISTS utxos (txid TEXT NOT NULL, vout INTEGER NOT NULL, value_sat INTEGER NOT NULL, address TEXT NOT NULL, spent INTEGER NOT NULL DEFAULT 0, PRIMARY KEY(txid, vout));")?;
        Ok(Self { db, network })
    }

    fn xpriv(&self) -> Result<Xpriv> {
        let value: String = self
            .db
            .query_row("SELECT xpriv FROM wallet WHERE id = 1", [], |row| {
                row.get(0)
            })
            .context("wallet is not initialized; run `init` first")?;
        Xpriv::from_str(&value).context("stored extended private key is invalid")
    }

    fn descriptors(&self) -> Result<(String, String)> {
        let xpriv = self.xpriv()?;
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let fingerprint = xpriv.fingerprint(&secp);
        let coin = if self.network == Network::Regtest {
            1
        } else {
            1
        };
        let origin = format!("[{fingerprint}/84'/{coin}'/0']");
        Ok((
            format!("wpkh({origin}{xpriv}/0/*)"),
            format!("wpkh({origin}{xpriv}/1/*)"),
        ))
    }

    fn address(&mut self, keychain: Keychain) -> Result<Address> {
        let xpriv = self.xpriv()?;
        let (column, branch, label) = match keychain {
            Keychain::External => ("next_external", 0, "external"),
            Keychain::Internal => ("next_internal", 1, "internal"),
        };
        let index: u32 = self.db.query_row(
            &format!("SELECT {column} FROM wallet WHERE id = 1"),
            [],
            |row| row.get(0),
        )?;
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let path = DerivationPath::from(vec![
            ChildNumber::from_normal_idx(branch)?,
            ChildNumber::from_normal_idx(index)?,
        ]);
        let child = xpriv.derive_priv(&secp, &path)?;
        let public = CompressedPublicKey::from_private_key(&secp, &child.to_priv())?;
        let address = Address::p2wpkh(&public, self.network);
        self.db.execute(
            "INSERT INTO addresses(address, keychain, child_index) VALUES (?1, ?2, ?3)",
            params![address.to_string(), label, index],
        )?;
        self.db.execute(
            &format!("UPDATE wallet SET {column} = {column} + 1 WHERE id = 1"),
            [],
        )?;
        Ok(address)
    }
}

fn rpc(cli: &Cli) -> Result<Client> {
    let auth = match (&cli.rpc_user, &cli.rpc_password) {
        (Some(user), Some(password)) => Auth::UserPass(user.clone(), password.clone()),
        _ => Auth::None,
    };
    Client::new(&cli.rpc_url, auth).context("connecting to Bitcoin Core RPC")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut wallet = Wallet::open(&cli.database, cli.network.bitcoin())?;
    match cli.command {
        Command::Init => init(&wallet)?,
        Command::Address { keychain } => println!("{}", wallet.address(keychain)?),
        Command::ShowDescriptor => {
            let (external, internal) = wallet.descriptors()?;
            println!("external: {external}\ninternal: {internal}");
        }
        Command::Balance => balance(&wallet)?,
        Command::Sync => sync(&mut wallet, &cli)?,
        Command::Send {
            ref address,
            amount_btc,
        } => send(&wallet, &cli, address, amount_btc)?,
    }
    Ok(())
}

fn init(wallet: &Wallet) -> Result<()> {
    if wallet
        .db
        .query_row::<String, _, _>("SELECT xpriv FROM wallet WHERE id = 1", [], |row| {
            row.get(0)
        })
        .is_ok()
    {
        bail!("wallet is already initialized")
    }
    let mut seed = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut seed);
    let xpriv = Xpriv::new_master(wallet.network, &seed)?;
    wallet.db.execute(
        "INSERT INTO wallet(id, xpriv) VALUES (1, ?1)",
        params![xpriv.to_string()],
    )?;
    println!("wallet initialized; private material is stored in the local database");
    Ok(())
}

fn balance(wallet: &Wallet) -> Result<()> {
    let sats: i64 = wallet.db.query_row(
        "SELECT COALESCE(SUM(value_sat), 0) FROM utxos WHERE spent = 0",
        [],
        |row| row.get(0),
    )?;
    println!("{sats} sats ({:.8} BTC)", sats as f64 / 100_000_000.0);
    Ok(())
}

fn sync(wallet: &mut Wallet, cli: &Cli) -> Result<()> {
    let client = rpc(cli)?;
    let (external, internal) = wallet.descriptors()?;
    let response = client
        .scan_tx_out_set_blocking(&[
            json::ScanTxOutRequest::Extended {
                desc: external,
                range: (0, 99),
            },
            json::ScanTxOutRequest::Extended {
                desc: internal,
                range: (0, 99),
            },
        ])
        .context("scanning the node UTXO set")?;
    wallet.db.execute("UPDATE utxos SET spent = 1", [])?;
    for utxo in response.unspents {
        wallet.db.execute(
            "INSERT INTO utxos(txid, vout, value_sat, address, spent) VALUES (?1, ?2, ?3, ?4, 0) ON CONFLICT(txid, vout) DO UPDATE SET value_sat=excluded.value_sat, address=excluded.address, spent=0",
            params![utxo.txid.to_string(), utxo.vout, utxo.amount.to_sat(), utxo.script_pub_key.to_hex_string()],
        )?;
    }
    let count: i64 = wallet
        .db
        .query_row("SELECT COUNT(*) FROM addresses", [], |row| row.get(0))?;
    client
        .get_blockchain_info()
        .context("querying node status")?;
    println!("connected to Bitcoin Core at {}", cli.rpc_url);
    println!("scanned the node UTXO set; {count} local addresses tracked");
    balance(wallet)?;
    Ok(())
}

fn send(wallet: &Wallet, cli: &Cli, address: &str, amount_btc: f64) -> Result<()> {
    let destination = Address::from_str(address)?.require_network(wallet.network)?;
    let amount = Amount::from_btc(amount_btc).context("amount must be a valid BTC value")?;
    let client = rpc(cli)?;
    let change = wallet
            .db
            .query_row::<String, _, _>(
                "SELECT address FROM addresses WHERE keychain = 'internal' ORDER BY child_index DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok()
            .and_then(|value| Address::from_str(&value).ok());
    let mut outputs = HashMap::new();
    outputs.insert(destination.to_string(), amount);
    let options = bitcoincore_rpc::json::WalletCreateFundedPsbtOptions {
        add_inputs: Some(true),
        change_address: change,
        include_watching: Some(true),
        ..Default::default()
    };
    let funded =
        client.wallet_create_funded_psbt(&[], &outputs, None, Some(options), Some(true))?;
    let signed = client.wallet_process_psbt(&funded.psbt, Some(true), None, Some(true))?;
    if !signed.complete {
        bail!("Bitcoin Core could not fully sign the funded PSBT")
    }
    let finalized = client.finalize_psbt(&signed.psbt, Some(true))?;
    if !finalized.complete {
        bail!("Bitcoin Core could not finalize the signed PSBT")
    }
    let raw = finalized
        .hex
        .context("Core returned no finalized transaction")?;
    let txid = client.send_raw_transaction(&raw)?;
    println!("broadcast transaction: {txid}");
    Ok(())
}
