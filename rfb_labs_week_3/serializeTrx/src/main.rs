use std::process::ExitCode;

use clap::Parser;
use serializetrx::{Cli, run};

fn main() -> ExitCode {
    // clap reports its own errors (unknown flag, missing --input) and exits.
    let cli = Cli::parse();

    match run(&cli) {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            // Validation failures go to stderr so a caller can pipe the hex on
            // stdout somewhere without picking up the noise.
            eprintln!("error: {error}");
            if let Some(hint) = error.hint() {
                eprintln!("hint:  {hint}");
            }
            ExitCode::FAILURE
        }
    }
}
