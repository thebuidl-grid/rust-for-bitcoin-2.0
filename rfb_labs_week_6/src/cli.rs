//! Command line surface.

use clap::{Args, Parser, Subcommand};

use crate::keys::DescriptorKind;
use crate::tx::Selection;

#[derive(Debug, Parser)]
#[command(
    name = "rfbwallet",
    about = "A descriptor wallet for Bitcoin regtest, built on rust-bitcoin, bitcoincore-rpc and BDK",
    version,
    max_term_width = 100
)]
pub struct Cli {
    /// Print debug logging from the wallet and its dependencies.
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create the wallet database and derive its descriptors.
    Init(InitArgs),

    /// Show the wallet's descriptors, keychain state and storage.
    Info,

    /// Reveal or inspect addresses.
    Address(AddressArgs),

    /// Show the confirmed and pending balance.
    Balance,

    /// List the wallet's unspent outputs.
    Utxos(UtxosArgs),

    /// List the wallet's transaction history.
    Txs,

    /// Pull new blocks and mempool entries from the node.
    Sync(SyncArgs),

    /// Build, sign and broadcast a payment.
    Send(SendArgs),

    /// Show what the configured Bitcoin node reports.
    Node,

    /// Mine regtest blocks, by default to this wallet.
    Mine(MineArgs),

    /// Decode a transaction and verify its signatures with rust-bitcoin.
    Verify(VerifyArgs),

    /// Compare the wpkh and tr descriptors derived from the same seed.
    Compare,

    /// Print a fresh BIP39 mnemonic without touching the wallet database.
    NewMnemonic,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Script type for both keychains.
    #[arg(long, value_enum, default_value_t = DescriptorKind::Wpkh)]
    pub descriptor: DescriptorKind,

    /// Record this height as the wallet birthday instead of asking the node.
    #[arg(long)]
    pub birthday: Option<u32>,
}

#[derive(Debug, Args)]
pub struct AddressArgs {
    /// Use the internal (change) keychain instead of the external one.
    #[arg(long)]
    pub change: bool,

    /// Show the address at this index without advancing the keychain.
    #[arg(long, conflicts_with = "unused")]
    pub peek: Option<u32>,

    /// Return the lowest revealed address that has not been used yet.
    #[arg(long)]
    pub unused: bool,
}

#[derive(Debug, Args)]
pub struct UtxosArgs {
    /// List every output instead of the largest few.
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Start emitting blocks from this height instead of the wallet birthday.
    #[arg(long)]
    pub from_height: Option<u32>,
}

#[derive(Debug, Args)]
pub struct SendArgs {
    /// Destination address.
    #[arg(long)]
    pub to: String,

    /// Amount in satoshis. Ignored when --drain is set.
    #[arg(long, default_value_t = 0)]
    pub amount: u64,

    /// Fee rate in sat/vB.
    #[arg(long, default_value_t = 2)]
    pub fee_rate: u64,

    /// Coin selection algorithm.
    #[arg(long, value_enum, default_value_t = Selection::Bnb)]
    pub selection: Selection,

    /// Send the whole spendable balance to --to.
    #[arg(long)]
    pub drain: bool,

    /// Build and sign, print the result, but do not broadcast.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct MineArgs {
    /// Number of blocks to mine.
    #[arg(long, default_value_t = 1)]
    pub blocks: u64,

    /// Mine to this address instead of a fresh wallet address.
    #[arg(long)]
    pub to: Option<String>,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Transaction id to decode and check.
    pub txid: String,
}
