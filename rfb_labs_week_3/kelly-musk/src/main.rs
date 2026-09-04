//! `txdecode` — command-line front end for the Week 3 transaction decoder.

use std::io::Read;
use std::process::ExitCode;

use clap::Parser;
use week3_txdecode::{decode_transaction_hex, parse_transaction_hex};

/// Decode a raw Bitcoin transaction (legacy or SegWit) into JSON.
#[derive(Parser, Debug)]
#[command(name = "txdecode", version, about)]
struct Cli {
    /// Raw transaction as a hex string (omit when using --file or --stdin).
    hex: Option<String>,

    /// Read the transaction hex from a file.
    #[arg(short, long, value_name = "FILE")]
    file: Option<String>,

    /// Read the transaction hex from standard input.
    #[arg(short, long)]
    stdin: bool,

    /// Emit compact single-line JSON instead of pretty-printed.
    #[arg(short, long)]
    compact: bool,

    /// Print only the txid and wtxid.
    #[arg(long)]
    txid: bool,
}

fn load_hex(cli: &Cli) -> Result<String, String> {
    let sources = [cli.hex.is_some(), cli.file.is_some(), cli.stdin]
        .into_iter()
        .filter(|set| *set)
        .count();
    if sources != 1 {
        return Err("provide exactly one of: HEX argument, --file, or --stdin".to_string());
    }

    if let Some(hex) = &cli.hex {
        Ok(hex.clone())
    } else if let Some(path) = &cli.file {
        std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))
    } else {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("reading stdin: {e}"))?;
        Ok(buf)
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let raw = load_hex(&cli)?;

    if cli.txid {
        let tx = parse_transaction_hex(&raw).map_err(|e| e.to_string())?;
        println!("txid  {}", tx.txid());
        println!("wtxid {}", tx.wtxid());
        return Ok(());
    }

    let decoded = decode_transaction_hex(&raw).map_err(|e| e.to_string())?;
    let json = if cli.compact {
        serde_json::to_string(&decoded)
    } else {
        serde_json::to_string_pretty(&decoded)
    }
    .map_err(|e| e.to_string())?;
    println!("{json}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
