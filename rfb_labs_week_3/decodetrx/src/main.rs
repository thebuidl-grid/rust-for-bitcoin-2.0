use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    // Define CLI using Clap

    // Retrieve transaction hex argument

    // Call the decoder function from the library
    let matches = Command::new("Transaction decoder")
        .version("1.0")
        .about("Bitcoin Transaction decoder")
        .arg(
            Arg::new("transaction_hex")
                .required(true)
                .help("(string, required) Raw Transaction hex"),
        )
        .get_matches();
    let transaction_hex = matches
        .get_one::<String>("transaction_hex")
        .expect("transaction hex is required")
        .to_string();

    match decode_transaction(transaction_hex) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("Error decodinng transaction: {err}"),
    }
}

// // https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2
