use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    // Define CLI using Clap
    let matches = Command::new("decodetrx")
        .version("1.0")
        .about("Decodes a raw Bitcoin transaction (legacy or SegWit)")
        .arg(
            Arg::new("transaction_hex")
                .help("Raw transaction hex, e.g. from mempool.space")
                .required(true)
                .index(1),
        )
        .get_matches();

    // Retrieve transaction hex argument
    let transaction_hex = matches
        .get_one::<String>("transaction_hex")
        .expect("transaction_hex is required")
        .to_owned();

    // Call the decoder function from the library
    match decode_transaction(transaction_hex) {
        Ok(decoded) => println!("{decoded}"),
        Err(error) => eprintln!("Failed to decode transaction: {error}"),
    }
}

// https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2
