//! Command-line interface: argument definitions and command implementations.
//!
//! `main.rs` stays thin — parse, dispatch here, print any error. Keeping the
//! presentation layer in one place means the library modules never print anything,
//! which is what lets the tests drive them directly.

use std::fs;
use std::path::Path;

use bdk_wallet::KeychainKind;
use bdk_wallet::bitcoin::{Amount, Network};
use bdk_wallet::keys::bip39::WordCount;
use clap::{Parser, Subcommand, ValueEnum};

use crate::config::{Config, restrict_permissions};
use crate::error::{Result, WalletError};
use crate::keys;
use crate::node;
use crate::tx::{self, Selection};
use crate::wallet::Wallet;

#[derive(Debug, Parser)]
#[command(
    name = "rfbwallet",
    version,
    about = "A descriptor-based Bitcoin wallet for regtest and testnet",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate a fresh seed phrase and write it to .env
    Init {
        /// Number of words in the generated mnemonic
        #[arg(long, value_enum, default_value_t = Words::W12)]
        words: Words,

        /// Overwrite an existing mnemonic. THIS DESTROYS THE OLD SEED.
        #[arg(long)]
        force: bool,
    },

    /// Show descriptors, fingerprint, network and database location
    Info,

    /// Receive and change addresses
    Address {
        #[command(subcommand)]
        action: AddressAction,
    },

    /// Show the wallet balance
    Balance,

    /// List unspent outputs
    Utxos,

    /// Pull blocks and mempool from the Bitcoin node
    Sync,

    /// Build, sign and broadcast a payment
    Send {
        /// Recipient address
        #[arg(long)]
        to: String,

        /// Amount in satoshis
        #[arg(long)]
        amount: u64,

        /// Fee rate in sat/vB
        #[arg(long, default_value_t = 2)]
        fee_rate: u64,

        /// Use largest-first coin selection instead of BDK's branch-and-bound
        #[arg(long)]
        largest_first: bool,

        /// Build and sign, print the details, but do not broadcast
        #[arg(long)]
        dry_run: bool,
    },

    /// Mine regtest blocks paying this wallet (regtest only)
    Fund {
        /// How many blocks to mine. 101 matures exactly one coinbase, since
        /// coinbase outputs need 100 confirmations before they can be spent.
        #[arg(long, default_value_t = 101)]
        blocks: u64,
    },
}

#[derive(Debug, Subcommand)]
pub enum AddressAction {
    /// Show an address to receive funds
    New {
        /// Use the internal (change) keychain instead of external
        #[arg(long)]
        change: bool,

        /// Always advance the index, even if the current address is unused.
        /// Without this, repeated calls return the same unused address.
        #[arg(long)]
        reveal: bool,
    },

    /// List every address revealed so far
    List,
}

/// Mnemonic lengths, as a clap-friendly enum.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Words {
    #[value(name = "12")]
    W12,
    #[value(name = "15")]
    W15,
    #[value(name = "18")]
    W18,
    #[value(name = "21")]
    W21,
    #[value(name = "24")]
    W24,
}

impl From<Words> for WordCount {
    fn from(w: Words) -> Self {
        match w {
            Words::W12 => WordCount::Words12,
            Words::W15 => WordCount::Words15,
            Words::W18 => WordCount::Words18,
            Words::W21 => WordCount::Words21,
            Words::W24 => WordCount::Words24,
        }
    }
}

/// Dispatch a parsed command.
pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { words, force } => init(words.into(), force),
        Command::Info => info(),
        Command::Address { action } => match action {
            AddressAction::New { change, reveal } => address_new(change, reveal),
            AddressAction::List => address_list(),
        },
        Command::Balance => balance(),
        Command::Utxos => utxos(),
        Command::Sync => sync(),
        Command::Fund { blocks } => fund(blocks),
        Command::Send {
            to,
            amount,
            fee_rate,
            largest_first,
            dry_run,
        } => send(&to, amount, fee_rate, largest_first, dry_run),
    }
}

// ---------------------------------------------------------------------------
// commands
// ---------------------------------------------------------------------------

/// Generate a seed and record it in `.env`.
///
/// The mnemonic is written to a gitignored file rather than printed alone, so a
/// closed terminal does not lose the wallet. It is also echoed once, because a seed
/// you have never seen is a seed you have not backed up.
fn init(words: WordCount, force: bool) -> Result<()> {
    let mnemonic = keys::generate_mnemonic(words)?;
    let phrase = mnemonic.to_string();

    let env_path = Path::new(".env");
    let created = scaffold_env(env_path, &phrase, force)?;

    println!();
    println!("  Generated a new {}-word seed phrase.", phrase.split_whitespace().count());
    println!();
    for (i, word) in phrase.split_whitespace().enumerate() {
        print!("{:>3}. {:<10}", i + 1, word);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }
    if phrase.split_whitespace().count() % 4 != 0 {
        println!();
    }
    println!();
    println!(
        "  {} .env  (permissions set to 0600)",
        if created { "Created" } else { "Updated" }
    );
    println!();
    println!("  Write these words down offline. Anyone holding them controls the wallet.");
    println!("  .env is gitignored — keep it that way, and never reuse this seed on mainnet.");
    println!();
    println!("  Next:  rfbwallet info");
    println!();

    Ok(())
}

/// Descriptors, identity, and where state lives.
fn info() -> Result<()> {
    let config = Config::load()?;
    let wallet = Wallet::open(&config)?;
    let d = wallet.descriptors();

    let revealed_ext = wallet
        .derivation_index(KeychainKind::External)
        .map_or("none".to_string(), |i| format!("0..={i}"));
    let revealed_int = wallet
        .derivation_index(KeychainKind::Internal)
        .map_or("none".to_string(), |i| format!("0..={i}"));

    println!();
    println!("  network         {}", wallet.network());
    println!("  script type     {} (BIP{})", d.kind, d.kind.purpose());
    println!("  account path    {}", d.account_path);
    println!("  fingerprint     {}", d.fingerprint);
    println!("  database        {}", config.db_path.display());
    println!("  node rpc        {}", config.rpc.url);
    println!();
    println!("  revealed external  {revealed_ext}");
    println!("  revealed internal  {revealed_int}");
    println!();
    println!("  Public descriptors (safe to share — watch-only, no spending power):");
    println!();
    println!("    external  {}", d.external_public);
    println!("    internal  {}", d.internal_public);
    println!();
    println!("  Private descriptors are never printed. They live in the seed (.env)");
    println!("  and in the wallet database.");
    println!();

    Ok(())
}

fn address_new(change: bool, reveal: bool) -> Result<()> {
    let config = Config::load()?;
    let mut wallet = Wallet::open(&config)?;

    let keychain = if change {
        KeychainKind::Internal
    } else {
        KeychainKind::External
    };

    // Default to `next_unused`: asking repeatedly should not burn through the gap
    // limit. `--reveal` forces the index forward.
    let info = if reveal {
        wallet.reveal_next_address(keychain)?
    } else {
        wallet.next_unused_address(keychain)?
    };

    println!();
    println!("  {}", info.address);
    println!();
    println!("  keychain  {}", keychain_name(keychain));
    println!("  index     {}", info.index);
    println!();

    Ok(())
}

fn address_list() -> Result<()> {
    let config = Config::load()?;
    let wallet = Wallet::open(&config)?;

    println!();
    for keychain in [KeychainKind::External, KeychainKind::Internal] {
        let addresses = wallet.revealed_addresses(keychain);
        println!("  {} ({})", keychain_name(keychain), addresses.len());

        if addresses.is_empty() {
            println!("    (none revealed yet)");
        }
        for a in addresses {
            println!("    {:>3}  {}", a.index, a.address);
        }
        println!();
    }

    Ok(())
}

fn balance() -> Result<()> {
    let config = Config::load()?;
    let wallet = Wallet::open(&config)?;
    let b = wallet.balance();

    println!();
    println!("  confirmed          {}", amount(b.confirmed));
    println!("  immature           {}", amount(b.immature));
    println!("  trusted pending    {}", amount(b.trusted_pending));
    println!("  untrusted pending  {}", amount(b.untrusted_pending));
    println!("  {}", "-".repeat(46));
    println!("  spendable          {}", amount(b.trusted_spendable()));
    println!("  total              {}", amount(b.total()));
    println!();

    if b.immature > Amount::ZERO {
        println!("  Immature coins are coinbase outputs awaiting 100 confirmations.");
        println!();
    }

    Ok(())
}

fn utxos() -> Result<()> {
    let config = Config::load()?;
    let wallet = Wallet::open(&config)?;
    let utxos = wallet.list_unspent();

    println!();
    if utxos.is_empty() {
        println!("  No unspent outputs. Run `rfbwallet sync` after funding the wallet.");
        println!();
        return Ok(());
    }

    for u in &utxos {
        let status = match u.chain_position.confirmation_height_upper_bound() {
            Some(h) => format!("confirmed @ {h}"),
            None => "unconfirmed".to_string(),
        };
        println!("  {}:{}", u.outpoint.txid, u.outpoint.vout);
        println!(
            "    {}   {:<9} index {:<4} {}",
            amount(u.txout.value),
            keychain_name(u.keychain),
            u.derivation_index,
            status
        );
    }
    println!();
    println!("  {} unspent output(s)", utxos.len());
    println!();

    Ok(())
}

fn sync() -> Result<()> {
    let config = Config::load()?;
    let mut wallet = Wallet::open(&config)?;

    println!();
    println!("  connecting to {} ...", config.rpc.url);
    let client = node::connect(&config)?;

    let before = wallet.balance().total();
    let report = node::sync(&mut wallet, &client)?;
    let after = wallet.balance().total();

    println!("  applied {} block(s), tip now {}", report.blocks_applied, report.tip_height);
    if report.mempool_txs > 0 {
        println!("  {} unconfirmed tx(s) in mempool", report.mempool_txs);
    }
    if report.evicted_txs > 0 {
        println!("  {} tx(s) evicted from mempool", report.evicted_txs);
    }
    println!();
    println!("  balance  {}", amount(after));
    if after != before {
        println!("  changed  {} -> {}", before.to_sat(), after.to_sat());
    }
    println!();

    Ok(())
}

fn fund(blocks: u64) -> Result<()> {
    let config = Config::load()?;

    // Mining on anything but regtest is either impossible or a very bad idea.
    if config.network != Network::Regtest {
        return Err(WalletError::InvalidEnv {
            key: "BITCOIN_NETWORK",
            value: config.network.to_string(),
            reason: "`fund` mines blocks, which only works on regtest. \
                     Use a faucet on signet or testnet",
        });
    }

    let mut wallet = Wallet::open(&config)?;
    let client = node::connect(&config)?;

    // Mine to our own external address so the coinbase outputs belong to us.
    let address = wallet.next_unused_address(KeychainKind::External)?;

    println!();
    println!("  mining {blocks} block(s) to {}", address.address);
    let hashes = node::mine_to(&client, &address.address, blocks)?;
    println!("  mined {} block(s), node tip now {}", hashes.len(), node::tip_height(&client)?);

    let report = node::sync(&mut wallet, &client)?;
    let b = wallet.balance();

    println!("  synced {} block(s)", report.blocks_applied);
    println!();
    println!("  spendable  {}", amount(b.trusted_spendable()));
    println!("  immature   {}", amount(b.immature));
    println!("  total      {}", amount(b.total()));
    println!();

    if b.immature > Amount::ZERO {
        let need = 100u64.saturating_sub(blocks.saturating_sub(1));
        println!("  Coinbase outputs mature after 100 confirmations.");
        if need > 0 {
            println!("  Mine ~{need} more block(s) to make them spendable.");
        }
        println!();
    }

    Ok(())
}

fn send(
    to: &str,
    amount_sat: u64,
    fee_rate_sat_vb: u64,
    largest_first: bool,
    dry_run: bool,
) -> Result<()> {
    let config = Config::load()?;
    let mut wallet = Wallet::open(&config)?;

    let recipient = tx::parse_address(&wallet, to)?;
    let value = Amount::from_sat(amount_sat);
    let fee_rate = tx::fee_rate_from_sat_per_vb(fee_rate_sat_vb)?;
    let selection = if largest_first {
        Selection::LargestFirst
    } else {
        Selection::Default
    };

    println!();
    println!("  paying    {} to {recipient}", amount(value));
    println!("  fee rate  {fee_rate_sat_vb} sat/vB");
    println!("  selection {}", if largest_first { "largest-first" } else { "branch-and-bound" });
    println!();

    let draft = tx::build_and_sign(&mut wallet, &recipient, value, fee_rate, selection)?;

    println!("  txid      {}", draft.txid);
    println!("  inputs    {}", draft.inputs);
    println!("  outputs   {}", draft.outputs);
    println!("  fee       {}", amount(draft.fee));
    match draft.change_vout {
        Some(vout) => println!(
            "  change    vout {vout}, {} -> internal keychain",
            amount(draft.change_amount)
        ),
        None => println!("  change    none (exact match, no change output)"),
    }
    println!("  size      {} vB", draft.tx.vsize());
    println!();

    if dry_run {
        println!("  --dry-run: signed but NOT broadcast.");
        println!();
        return Ok(());
    }

    let client = node::connect(&config)?;
    let txid = tx::broadcast(&client, &draft.tx)?;

    println!("  broadcast ok");
    println!("  txid      {txid}");
    println!();
    println!("  Verify:  bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser \\");
    println!("             -rpcpassword=polarpass getrawtransaction {txid} true");
    println!();

    // Pick the transaction up from the mempool so it shows as pending right away.
    let report = node::sync(&mut wallet, &client)?;
    if report.mempool_txs > 0 {
        println!("  now in mempool; balance {}", amount(wallet.balance().total()));
        println!();
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn keychain_name(k: KeychainKind) -> &'static str {
    match k {
        KeychainKind::External => "external",
        KeychainKind::Internal => "internal",
    }
}

/// Render an amount in both BTC and satoshis. Satoshis are the unit you actually
/// reason about when checking fees, so showing both avoids decimal-point mistakes.
fn amount(a: Amount) -> String {
    format!("{:>14.8} BTC  ({:>12} sat)", a.to_btc(), a.to_sat())
}

/// Write the mnemonic into `.env`, creating the file if needed.
///
/// Returns `true` if the file was created, `false` if an existing one was updated.
/// Refuses to clobber an existing seed unless `force` is set — overwriting a seed
/// that still holds coins is unrecoverable.
fn scaffold_env(path: &Path, mnemonic: &str, force: bool) -> Result<bool> {
    const KEY: &str = "WALLET_MNEMONIC=";

    if !path.exists() {
        fs::write(path, env_template(mnemonic))?;
        restrict_permissions(path)?;
        return Ok(true);
    }

    let existing = fs::read_to_string(path)?;
    let has_seed = existing.lines().any(|line| {
        line.trim_start()
            .strip_prefix(KEY)
            // An empty *or* empty-quoted value (`WALLET_MNEMONIC=""`) is a
            // placeholder, not a seed, and may be filled in without --force.
            .is_some_and(|rest| !rest.trim().trim_matches('"').trim().is_empty())
    });

    if has_seed && !force {
        return Err(WalletError::InvalidEnv {
            key: ".env",
            value: path.display().to_string(),
            reason: "already contains a seed phrase. Pass --force to replace it, \
                     but back up the old one first — this is not reversible",
        });
    }

    // Rewrite in place so any other settings the user has customised survive.
    let mut out = String::new();
    let mut replaced = false;
    for line in existing.lines() {
        if line.trim_start().starts_with(KEY) {
            out.push_str(&seed_line(mnemonic));
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        out.push_str(&seed_line(mnemonic));
    }

    fs::write(path, out)?;
    restrict_permissions(path)?;
    Ok(false)
}

/// The `WALLET_MNEMONIC` line, with the value quoted.
///
/// The quotes are load-bearing. A seed phrase is words separated by spaces, and
/// dotenv parsers treat an unquoted value as ending at the first space — dotenvy
/// rejects the line outright rather than silently truncating. Every writer of this
/// key must go through here.
fn seed_line(mnemonic: &str) -> String {
    format!("WALLET_MNEMONIC=\"{mnemonic}\"\n")
}

fn env_template(mnemonic: &str) -> String {
    format!(
        "# Generated by `rfbwallet init`. Gitignored — never commit this file.\n\
         \n\
         # Network: regtest | signet | testnet | testnet4. Mainnet is refused.\n\
         BITCOIN_NETWORK=regtest\n\
         \n\
         # BIP39 seed phrase. Anyone holding this controls the wallet.\n\
         # The quotes matter: the value contains spaces.\n\
         WALLET_MNEMONIC=\"{mnemonic}\"\n\
         \n\
         # Optional BIP39 passphrase (the \"25th word\"). Changing it changes the wallet.\n\
         WALLET_PASSPHRASE=\n\
         \n\
         # Wallet state. Contains private descriptors — gitignored.\n\
         WALLET_DB=./wallet.sqlite\n\
         \n\
         # Script type for both keychains: wpkh (BIP84) | tr (BIP86 Taproot)\n\
         DESCRIPTOR_KIND=wpkh\n\
         \n\
         # Bitcoin Core RPC. Defaults suit a Polar regtest node.\n\
         RPC_URL=http://127.0.0.1:18443\n\
         RPC_USER=polaruser\n\
         RPC_PASSWORD=polarpass\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rfbw-env-{tag}-{}-{nanos}", std::process::id()))
    }

    /// Parse a written .env the way `Config::load` will, without touching the
    /// process environment (these tests run in parallel).
    fn parse_env(path: &Path) -> std::collections::HashMap<String, String> {
        dotenvy::from_path_iter(path)
            .expect("dotenvy could not open the file")
            .map(|entry| entry.expect("dotenvy could not parse a line"))
            .collect()
    }

    #[test]
    fn creates_a_new_env_file() {
        let p = temp_path("create");
        assert!(scaffold_env(&p, "word word word", false).unwrap());

        let contents = fs::read_to_string(&p).unwrap();
        assert!(contents.contains("BITCOIN_NETWORK=regtest"));

        let _ = fs::remove_file(&p);
    }

    /// A seed phrase is words separated by spaces, and an unquoted dotenv value
    /// ends at the first space. Asserting the file merely *contains* the text is
    /// not enough — it has to round-trip through the parser that will read it.
    #[test]
    fn written_env_round_trips_through_dotenvy() {
        let p = temp_path("roundtrip");
        // The canonical BIP39 test vector, published in the spec. Never use a real
        // generated seed as a fixture, even a throwaway regtest one.
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        scaffold_env(&p, seed, false).unwrap();

        let parsed = parse_env(&p);
        assert_eq!(parsed.get("WALLET_MNEMONIC").map(String::as_str), Some(seed));
        assert_eq!(parsed.get("BITCOIN_NETWORK").map(String::as_str), Some("regtest"));

        let _ = fs::remove_file(&p);
    }

    #[test]
    fn in_place_rewrite_also_round_trips() {
        let p = temp_path("roundtrip2");
        fs::write(&p, "BITCOIN_NETWORK=regtest\nWALLET_MNEMONIC=\"\"\nDESCRIPTOR_KIND=tr\n")
            .unwrap();

        let seed = "one two three four five six seven eight nine ten eleven twelve";
        assert!(!scaffold_env(&p, seed, false).unwrap());

        let parsed = parse_env(&p);
        assert_eq!(parsed.get("WALLET_MNEMONIC").map(String::as_str), Some(seed));
        assert_eq!(parsed.get("DESCRIPTOR_KIND").map(String::as_str), Some("tr"));

        let _ = fs::remove_file(&p);
    }

    #[test]
    fn refuses_to_clobber_an_existing_seed() {
        let p = temp_path("clobber");
        fs::write(&p, "WALLET_MNEMONIC=original seed here\nDESCRIPTOR_KIND=tr\n").unwrap();

        let err = scaffold_env(&p, "replacement", false).unwrap_err();
        assert!(matches!(err, WalletError::InvalidEnv { .. }));

        // The original must be untouched.
        let contents = fs::read_to_string(&p).unwrap();
        assert!(contents.contains("original seed here"));

        let _ = fs::remove_file(&p);
    }

    #[test]
    fn force_replaces_the_seed_but_keeps_other_settings() {
        let p = temp_path("force");
        fs::write(&p, "WALLET_MNEMONIC=old\nDESCRIPTOR_KIND=tr\nWALLET_DB=./custom.sqlite\n")
            .unwrap();

        assert!(!scaffold_env(&p, "brand new seed", true).unwrap());

        let contents = fs::read_to_string(&p).unwrap();
        assert!(contents.contains(r#"WALLET_MNEMONIC="brand new seed""#));
        assert!(!contents.contains("old"));
        // Customised settings survive the rewrite.
        assert!(contents.contains("DESCRIPTOR_KIND=tr"));
        assert!(contents.contains("WALLET_DB=./custom.sqlite"));

        let _ = fs::remove_file(&p);
    }

    #[test]
    fn fills_in_an_empty_placeholder_without_force() {
        let p = temp_path("placeholder");
        fs::write(&p, "BITCOIN_NETWORK=regtest\nWALLET_MNEMONIC=\n").unwrap();

        // An empty value is not a seed, so no --force needed.
        assert!(!scaffold_env(&p, "fresh seed", false).unwrap());
        assert!(fs::read_to_string(&p).unwrap().contains(r#"WALLET_MNEMONIC="fresh seed""#));

        let _ = fs::remove_file(&p);
    }

    #[cfg(unix)]
    #[test]
    fn env_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let p = temp_path("perms");
        scaffold_env(&p, "some seed", false).unwrap();

        let mode = fs::metadata(&p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "seed file must not be group/world readable");

        let _ = fs::remove_file(&p);
    }

    #[test]
    fn cli_parses() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
