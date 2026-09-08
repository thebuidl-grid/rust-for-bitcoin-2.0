//! `rfbwallet` — a descriptor-based Bitcoin wallet for regtest and testnet.
//!
//! Deliberately thin. Parse arguments, hand off to [`cli::run`], and turn any error
//! into a readable message plus a non-zero exit code. Nothing below this file
//! prints, and nothing panics on bad input.

use std::process::ExitCode;

use clap::Parser;
use rfb_labs_week_6::cli::{self, Cli};

fn main() -> ExitCode {
    // clap handles --help/--version and exits on its own for malformed arguments.
    let cli = Cli::parse();

    match cli::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!();
            eprintln!("  error: {e}");

            // Walk the source chain. thiserror's `#[from]` preserves the underlying
            // cause, and for RPC or SQLite failures the detail is usually one or two
            // levels down.
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                eprintln!("    caused by: {cause}");
                source = cause.source();
            }
            eprintln!();

            ExitCode::FAILURE
        }
    }
}
