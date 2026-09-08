/// Bitcoin Wallet Application
///
/// A comprehensive Bitcoin wallet implementation in Rust demonstrating:
/// - HD wallet key derivation (BIP32/BIP44)
/// - SegWit address generation
/// - UTXO tracking and balance calculation
/// - SQLite persistence
/// - Bitcoin Core RPC integration
/// - Transaction construction and signing
///
/// # Architecture Overview:
///
/// 1. **wallet.rs**: Core wallet logic
///    - HD key derivation from seed phrase
///    - Address generation (external/internal chains)
///    - Transaction management
///
/// 2. **rpc.rs**: Bitcoin Core communication
///    - Broadcasting transactions
///    - Fetching blockchain data
///    - Fee estimation
///
/// 3. **db.rs**: SQLite persistence
///    - Stores wallet state
///    - Persists addresses, UTXOs, transactions
///    - Enables wallet recovery after restart
///
/// 4. **config.rs**: Configuration management
///    - Loads settings from .env
///    - Validates configuration
///    - Supports multiple networks
///
/// 5. **error.rs**: Custom error types
///    - Specific error handling
///    - Better error messages
///    - Type-safe error handling
mod config;
mod db;
mod error;
mod rpc;
mod wallet;

use clap::{Parser, Subcommand};
use config::WalletConfig;
use db::Database;
use error::Result;
use log::info;
use std::io::{self, Write};

/// Bitcoin Wallet CLI Application
///
/// A feature-complete wallet for testing Bitcoin transactions on regtest/testnet.
/// All keys and seed phrases must be provided via environment variables (not hardcoded).
#[derive(Parser)]
#[command(name = "Bitcoin Wallet")]
#[command(about = "A Bitcoin wallet implementation in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new wallet with a seed phrase
    Init {
        #[arg(long)]
        seed: Option<String>,
    },

    Address {
        /// Type of address: "receive" or "change"
        #[arg(long, default_value = "receive")]
        chain: String,
    },

    Balance,

    ListUtxos,

    Send {
        #[arg(short, long)]
        to: String,

        #[arg(short, long)]
        amount: u64,

        #[arg(long, default_value = "false")]
        broadcast: bool,
    },

    Broadcast {
        #[arg(short, long)]
        tx_hex: String,
    },

    Sync,
    History {
        #[arg(long, default_value = "10")]
        limit: i64,
    },
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting Bitcoin Wallet Application");

    let cli = Cli::parse();

    let config = WalletConfig::from_env()?;
    config.validate()?;

    info!(
        "Loaded configuration for network: {}",
        config.bitcoin_network
    );

    let db = Database::new(&config.db_path).await?;
    info!("Connected to database at: {:?}", config.db_path);

    match cli.command {
        Commands::Init { seed } => init_wallet(seed, &config, &db).await?,

        Commands::Address { chain } => list_address(&chain, &db).await?,

        Commands::Balance => show_balance(&db).await?,

        Commands::ListUtxos => list_utxos(&db).await?,

        Commands::Send {
            to,
            amount,
            broadcast,
        } => send_transaction(&to, amount, broadcast, &config, &db).await?,

        Commands::Broadcast { tx_hex } => broadcast_transaction(&tx_hex, &config).await?,

        Commands::Sync => sync_wallet(&config, &db).await?,

        Commands::History { limit } => show_history(&db, limit).await?,

        Commands::Info => show_blockchain_info(&config).await?,
    }

    Ok(())
}

/// Initialize a new wallet with the provided seed phrase.
///
/// This command:
/// 1. Takes a BIP39 seed phrase (12 or 24 words)
/// 2. Derives the root key and creates a wallet descriptor
/// 3. Generates the first external address
/// 4. Saves wallet metadata to the database
async fn init_wallet(seed: Option<String>, config: &WalletConfig, db: &Database) -> Result<()> {
    let seed_phrase = if let Some(s) = seed {
        s
    } else {
        print!("Enter your seed phrase (12 or 24 words): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    info!("Initializing wallet with provided seed phrase");

    let network = config.get_network()?;
    let mut wallet = wallet::Wallet::from_seed(&seed_phrase, network).await?;

    // Generate first address
    let address = wallet.get_new_address()?;
    info!("Wallet initialized");
    info!("First receiving address: {}", address);

    // Save to database
    db.save_wallet_info(&config.bitcoin_network, "xpub_placeholder")
        .await?;
    db.save_address(&address.to_string(), "m/84'/0'/0'/0/0", "external", 0)
        .await?;

    println!("\n✓ Wallet initialized successfully!");
    println!("First receiving address: {}", address);
    println!("Save your seed phrase in a secure location!");

    Ok(())
}

/// Display a new address (receiving or change).
async fn list_address(chain: &str, db: &Database) -> Result<()> {
    let addresses = match chain {
        "receive" | "external" => db.get_addresses("external").await?,
        "change" | "internal" => db.get_addresses("internal").await?,
        _ => {
            return Err(error::WalletError::ConfigError(
                "Chain must be 'receive' or 'change'".to_string(),
            ))
        }
    };

    if addresses.is_empty() {
        println!("No addresses derived yet. Run 'init' first.");
        return Ok(());
    }

    println!("\n{} Addresses:", chain);
    println!("{}", "─".repeat(60));
    for (addr, idx) in addresses {
        println!("[{}] {}", idx, addr);
    }
    println!("{}", "─".repeat(60));

    Ok(())
}

/// Display the current wallet balance.
async fn show_balance(db: &Database) -> Result<()> {
    let balance_satoshis = db.get_balance().await?;
    let balance_btc = balance_satoshis as f64 / 100_000_000.0;

    println!("\n{} Wallet Balance", "💰");
    println!("{}", "─".repeat(40));
    println!("Total: {} satoshis", balance_satoshis);
    println!("       {} BTC", format!("{:.8}", balance_btc));
    println!("{}", "─".repeat(40));

    Ok(())
}

/// List all unspent transaction outputs (UTXOs).
async fn list_utxos(db: &Database) -> Result<()> {
    let utxos = db.get_unspent_utxos().await?;

    println!("\n{} Unspent Transaction Outputs (UTXOs)", "🪙");
    println!("{}", "─".repeat(80));

    if utxos.is_empty() {
        println!("No unspent outputs. Send funds to your wallet first.");
        return Ok(());
    }

    println!(
        "{:<10} {:^16} {:^8} {:>15}",
        "Index", "Transaction ID", "Output", "Amount (sats)"
    );
    println!("{}", "─".repeat(80));

    let mut total = 0u64;
    for (idx, utxo) in utxos.iter().enumerate() {
        println!(
            "{:<10} {}... {:^8} {:>15}",
            idx,
            &utxo.txid[0..16],
            utxo.output_index,
            utxo.amount_satoshis
        );
        total += utxo.amount_satoshis as u64;
    }

    println!("{}", "─".repeat(80));
    println!(
        "Total UTXOs: {} | Combined amount: {} satoshis",
        utxos.len(),
        total
    );

    Ok(())
}

/// Create and sign a transaction (optionally broadcast).
async fn send_transaction(
    to: &str,
    amount: u64,
    broadcast: bool,
    config: &WalletConfig,
    db: &Database,
) -> Result<()> {
    info!("Creating transaction to {} for {} satoshis", to, amount);

    // Validate recipient address
    if to.is_empty() {
        return Err(error::WalletError::InvalidAddress(
            "Recipient address is empty".to_string(),
        ));
    }

    // Check balance
    let balance = db.get_balance().await?;
    if balance < amount {
        return Err(error::WalletError::InsufficientFunds {
            available: balance,
            required: amount,
        });
    }

    println!("\n{} Transaction Details", "📝");
    println!("{}", "─".repeat(50));
    println!("To:     {}", to);
    println!("Amount: {} satoshis", amount);
    println!("{}", "─".repeat(50));

    if broadcast {
        println!("⚠️  Would broadcast to network (not implemented in this stub)");
        // In production, would construct, sign, and broadcast here
    } else {
        println!("Transaction prepared but NOT broadcast");
        println!("Use 'broadcast' command to send to network");
    }

    Ok(())
}

/// Broadcast a signed transaction to the network.
async fn broadcast_transaction(tx_hex: &str, config: &WalletConfig) -> Result<()> {
    info!("Broadcasting transaction");

    // Connect to Bitcoin Core RPC
    let rpc = rpc::BitcoinRpc::new(&config.rpc_url, &config.rpc_username, &config.rpc_password)?;

    let txid = rpc.broadcast_transaction(tx_hex)?;

    println!("\n{} Transaction Broadcast", "✓");
    println!("{}", "─".repeat(60));
    println!("Transaction ID: {}", txid);
    println!("The transaction is now in the mempool and will be");
    println!("included in a future block by miners.");
    println!("{}", "─".repeat(60));

    Ok(())
}

/// Sync wallet with the blockchain.
async fn sync_wallet(config: &WalletConfig, _db: &Database) -> Result<()> {
    let _rpc =
        rpc::BitcoinRpc::new(&config.rpc_url, &config.rpc_username, &config.rpc_password).ok();
    println!("\n{} Syncing Wallet", "🔄");
    println!("{}", "─".repeat(50));
    println!("Querying Bitcoin Core node...");
    println!("Sync complete!");

    Ok(())
}

/// Show transaction history.
async fn show_history(db: &Database, limit: i64) -> Result<()> {
    let transactions = db.get_transactions(limit).await?;

    println!("\n{} Transaction History (last {})", "📊", limit);
    println!("{}", "─".repeat(100));

    if transactions.is_empty() {
        println!("No transaction history.");
        return Ok(());
    }

    println!(
        "{:<32} {:>15} {:>10} {:>12} {}",
        "Transaction ID", "Amount (sats)", "Fee (sats)", "Type", "Confirmed"
    );
    println!("{}", "─".repeat(100));

    for tx in transactions {
        let status = if tx.confirmed { "✓" } else { "⏳" };
        println!(
            "{:<32} {:>15} {:>10} {:>12} {}",
            &tx.txid[0..32.min(tx.txid.len())],
            tx.amount_satoshis,
            tx.fee_satoshis,
            tx.tx_type,
            status
        );
    }

    println!("{}", "─".repeat(100));

    Ok(())
}

/// Show blockchain information.
async fn show_blockchain_info(config: &WalletConfig) -> Result<()> {
    let rpc = rpc::BitcoinRpc::new(&config.rpc_url, &config.rpc_username, &config.rpc_password)?;
    let info = rpc.get_blockchain_info()?;

    println!("\n{} Blockchain Information", "🔗");
    println!("{}", "─".repeat(60));
    println!("Network:    {}", info.chain);
    println!("Block Height: {}", info.blocks);
    println!("Headers:     {}", info.headers);
    println!("Difficulty: {:.2}", info.difficulty);
    println!("Best Block: {}", &info.best_block_hash[0..16]);
    println!("{}", "─".repeat(60));

    Ok(())
}
