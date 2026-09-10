use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    let matches = Command::new("Transaction Decoder")
        .version("1.0")
        .about("Bitcoin Transaction Decoder")
        .arg(
            Arg::new("transaction")
                .required(true)
                .help("Raw transaction hex")
        )
        .get_matches();

    let transaction_hex = matches.get_one::<String>("transaction").unwrap().clone();

    match decode_transaction(transaction_hex) {
        Ok(decoded) => println!("{}", decoded),
        Err(e) => eprintln!("Error decoding transaction: {}", e),
    }
}

// https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2