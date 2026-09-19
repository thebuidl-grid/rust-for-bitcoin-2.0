mod cli;
mod transaction;

use clap::Parser;

use cli::{build_transaction, Cli};
use transaction::{bytes_to_hex, serialize_transaction};

fn main() {
    let args = Cli::parse();

    let trx = match build_transaction(&args) {
        Ok(trx) => trx,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let serialized = serialize_transaction(&trx);

    println!("Serialized transaction:");
    println!("{:?}", &serialized);
    println!("Serialized Hex transaction:");
    println!("{}", bytes_to_hex(&serialized));

    println!("\nTransaction size: {} bytes", serialized.len());
}
