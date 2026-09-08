use std::process::ExitCode;

use clap::Parser;
use decodetrx::{decode_transaction, error::DecodeError};

/// Decode a raw Bitcoin transaction supplied as hexadecimal text.
#[derive(Debug, Parser)]
#[command(name = "decodetrx", version, about)]
struct Cli {
    /// Complete raw transaction hex
    transaction_hex: String,
}

fn run() -> Result<(), DecodeError> {
    let cli = Cli::parse();
    let decoded_transaction = decode_transaction(cli.transaction_hex)?;

    println!("{decoded_transaction}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

// // https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2
