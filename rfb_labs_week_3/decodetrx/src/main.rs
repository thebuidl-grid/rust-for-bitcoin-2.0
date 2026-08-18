use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    // Define CLI using Clap
    let matches = Command::new("Transaction decoder")
        .version("1.0")
        .about("Bitcoin Transaction decoder")
        .arg(
            Arg::new("transaction_hex")
                .required(true)
                .help("(string, required) Raw transaction hex"),
        )
        .get_matches();

    // Retrieve transaction hex argument
    let transaction_hex = matches.get_one::<String>("transaction_hex").unwrap();

    // Call the decoder function from the library
    match decode_transaction(transaction_hex.to_string()) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error: {}", e),
    }
}

// // https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2
