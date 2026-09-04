// House style for this crate: explicit `return` everywhere, so silence the lint.
#![allow(clippy::needless_return)]

use anyhow::Context;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use rfb_labs_week_6::config::{self, Config};
use rfb_labs_week_6::node::Node;
use rfb_labs_week_6::wallet::Wallet;
use rfb_labs_week_6::{raw_demo, sync, tx};

/// A minimal regtest / testnet Bitcoin wallet.
///
/// Wallet logic runs through BDK; node I/O runs through bitcoincore-rpc; raw
/// primitives come from rust-bitcoin. Configuration is read from `.env`.
#[derive(Parser, Debug)]
#[command(name = "rfbwallet", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create the wallet: generate (or reuse) a mnemonic, derive descriptors,
    /// and open a fresh SQLite database.
    Init {
        /// Use a Taproot (`tr`, BIP86) descriptor instead of `wpkh` (BIP84).
        #[arg(long)]
        taproot: bool,
    },
    /// Reveal a new receiving (external keychain) address.
    Address,
    /// Show confirmed and pending balance.
    Balance,
    /// List tracked unspent outputs.
    Utxos,
    /// Sync wallet state from the node (blocks + mempool).
    Sync,
    /// Build, sign and broadcast a payment.
    Send {
        /// Destination address.
        #[arg(long)]
        to: String,
        /// Amount in satoshis (ignored with --drain).
        #[arg(long, default_value_t = 0)]
        amount_sat: u64,
        /// Fee rate in sat/vB. Defaults to the node's estimate, or 1 sat/vB.
        #[arg(long)]
        fee_rate: Option<u64>,
        /// Restrict spending to these outpoints (`txid:vout`). Repeatable.
        #[arg(long = "utxo")]
        utxos: Vec<String>,
        /// Send the entire spendable balance to --to.
        #[arg(long)]
        drain: bool,
    },
    /// Print node chain info (proves the RPC connection works).
    NodeInfo,
    /// Regtest only: mine blocks to the wallet (or to --to).
    Mine {
        /// Number of blocks to mine.
        blocks: u64,
        /// Address to mine to. Defaults to a wallet receiving address.
        #[arg(long)]
        to: Option<String>,
    },
    /// Print the wallet's public descriptors (external and internal).
    Descriptors,
    /// Stretch goal: raw rust-bitcoin. Decode a tx and build an OP_RETURN.
    RawDemo {
        /// Optional raw transaction hex to decode.
        #[arg(long)]
        decode: Option<String>,
        /// Message to embed in the demo OP_RETURN output.
        #[arg(long, default_value = "rfb-week6")]
        message: String,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    return match cli.command {
        Command::Init { taproot } => run_init(taproot),
        Command::Address => run_address(),
        Command::Balance => run_balance(),
        Command::Utxos => run_utxos(),
        Command::Sync => run_sync(),
        Command::Send {
            to,
            amount_sat,
            fee_rate,
            utxos,
            drain,
        } => run_send(to, amount_sat, fee_rate, utxos, drain),
        Command::NodeInfo => run_node_info(),
        Command::Mine { blocks, to } => run_mine(blocks, to),
        Command::Descriptors => run_descriptors(),
        Command::RawDemo { decode, message } => run_raw_demo(decode, message),
    };
}

// === Commands

fn run_init(taproot: bool) -> anyhow::Result<()> {
    let (env_path, generated) =
        config::bootstrap_env(taproot).context("preparing .env for a new wallet")?;
    if generated {
        println!(
            "Generated a new mnemonic and wrote it to {}",
            env_path.display()
        );
    } else {
        println!("Using the existing MNEMONIC from {}", env_path.display());
    }

    let config = Config::load()?;
    let mut wallet = Wallet::create(&config).context("creating the wallet database")?;

    let address = wallet.new_receive_address()?;
    println!("Wallet created at {}", config.db_path.display());
    println!("Network:      {}", config.network);
    println!("Descriptor:   {}", config.descriptor_kind.as_str());
    println!(
        "First receive address ({}): {}",
        address.index, address.address
    );
    println!("\nNext: fund it, then run `rfbwallet sync` and `rfbwallet balance`.");
    return Ok(());
}

fn run_address() -> anyhow::Result<()> {
    let config = Config::load()?;
    let mut wallet = Wallet::open(&config)?;
    let address = wallet.new_receive_address()?;
    println!("{} (index {})", address.address, address.index);
    return Ok(());
}

fn run_balance() -> anyhow::Result<()> {
    let config = Config::load()?;
    let wallet = Wallet::open(&config)?;
    let balance = wallet.balance();
    println!("confirmed:          {}", balance.confirmed);
    println!("pending (incoming): {}", balance.untrusted_pending);
    println!("pending (change):   {}", balance.trusted_pending);
    println!("immature (coinbase):{}", balance.immature);
    println!("---");
    println!("total:              {}", balance.total());
    println!("spendable now:      {}", balance.trusted_spendable());
    return Ok(());
}

fn run_utxos() -> anyhow::Result<()> {
    let config = Config::load()?;
    let wallet = Wallet::open(&config)?;
    let utxos = wallet.utxos();
    if utxos.is_empty() {
        println!("no unspent outputs (try `rfbwallet sync`)");
        return Ok(());
    }
    for utxo in utxos {
        println!(
            "{}:{:<3} {:>15} sat  {:<8} {}",
            utxo.outpoint.txid,
            utxo.outpoint.vout,
            utxo.value.to_sat(),
            format!("{:?}", utxo.keychain).to_lowercase(),
            if utxo.confirmed {
                "confirmed"
            } else {
                "pending"
            },
        );
    }
    return Ok(());
}

fn run_sync() -> anyhow::Result<()> {
    let config = Config::load()?;
    let node = Node::connect(&config)?;
    let mut wallet = Wallet::open(&config)?;

    println!("syncing from height {} ...", config.start_height);
    let report = sync::run(&mut wallet, &node, config.start_height)?;
    println!(
        "applied {} block(s); wallet tip {}:{}",
        report.blocks_applied, report.tip_height, report.tip_hash
    );
    println!("balance now: {}", wallet.balance().total());
    return Ok(());
}

fn run_send(
    to: String,
    amount_sat: u64,
    fee_rate: Option<u64>,
    utxos: Vec<String>,
    drain: bool,
) -> anyhow::Result<()> {
    if !drain && amount_sat == 0 {
        anyhow::bail!("provide --amount-sat, or use --drain to send everything");
    }

    let config = Config::load()?;
    let node = Node::connect(&config)?;
    let mut wallet = Wallet::open(&config)?;

    let request =
        tx::SpendRequest::parse(&to, amount_sat, fee_rate, &utxos, drain, config.network)?;
    let outcome = tx::send(&mut wallet, &node, request)?;

    println!("broadcast ok");
    println!("txid: {}", outcome.txid);
    println!("sent: {}", outcome.sent);
    println!("fee:  {}", outcome.fee);
    println!("raw:  {}", outcome.raw_hex);
    println!(
        "\nverify with: bitcoin-core.cli -{} getrawtransaction {} true",
        core_net_flag(config.network),
        outcome.txid
    );
    return Ok(());
}

fn run_node_info() -> anyhow::Result<()> {
    let config = Config::load()?;
    let node = Node::connect(&config)?;
    let info = node.blockchain_info()?;
    println!("chain:               {}", info.chain);
    println!("blocks:              {}", info.blocks);
    println!("headers:             {}", info.headers);
    println!("best block hash:     {}", info.best_block_hash);
    println!("verification prog.:  {:.4}", info.verification_progress);
    println!("initial block dl:    {}", info.initial_block_download);
    return Ok(());
}

fn run_mine(blocks: u64, to: Option<String>) -> anyhow::Result<()> {
    let config = Config::load()?;
    let node = Node::connect(&config)?;
    let mut wallet = Wallet::open(&config)?;

    let address = match to {
        Some(raw) => bdk_wallet::bitcoin::Address::from_str_checked(&raw, config.network)
            .map_err(|e| anyhow::anyhow!("bad --to address: {e}"))?,
        None => wallet.new_receive_address()?.address,
    };

    let hashes = node.mine(blocks, &address)?;
    println!("mined {} block(s) to {}", hashes.len(), address);
    if let Some(last) = hashes.last() {
        println!("tip: {last}");
    }
    println!("run `rfbwallet sync` to pick up the new coins");
    return Ok(());
}

fn run_descriptors() -> anyhow::Result<()> {
    let config = Config::load()?;
    let wallet = Wallet::open(&config)?;
    let (external, internal) = wallet.public_descriptors();
    println!("external (receive): {external}");
    println!("internal (change):  {internal}");
    return Ok(());
}

fn run_raw_demo(decode: Option<String>, message: String) -> anyhow::Result<()> {
    if let Some(hex) = decode {
        println!("{}", raw_demo::decode(&hex)?);
    }
    let (txid, hex) = raw_demo::build_op_return(&message)?;
    println!("hand-built OP_RETURN transaction:");
    println!("  txid: {txid}");
    println!("  hex:  {hex}");
    return Ok(());
}

// === Helpers

fn core_net_flag(network: bdk_wallet::bitcoin::Network) -> &'static str {
    return match network {
        bdk_wallet::bitcoin::Network::Testnet => "testnet",
        bdk_wallet::bitcoin::Network::Signet => "signet",
        _ => "regtest",
    };
}

// A tiny helper: `Address::from_str` then `require_network`, used by `mine`.
trait AddressFromStrChecked: Sized {
    fn from_str_checked(raw: &str, network: bdk_wallet::bitcoin::Network) -> Result<Self, String>;
}

impl AddressFromStrChecked for bdk_wallet::bitcoin::Address {
    fn from_str_checked(raw: &str, network: bdk_wallet::bitcoin::Network) -> Result<Self, String> {
        use std::str::FromStr;
        let unchecked = bdk_wallet::bitcoin::Address::from_str(raw).map_err(|e| e.to_string())?;
        return unchecked
            .require_network(network)
            .map_err(|e| e.to_string());
    }
}
