use bitcoin_tx_serializer::{Args, run};
use clap::Parser;
use std::process;

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(result) => {
            println!("{}", result);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
