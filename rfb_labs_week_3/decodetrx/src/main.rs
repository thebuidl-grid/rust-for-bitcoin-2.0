use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    // Define CLI using Clap
    let matches = Command::new("Transaction Decoder")
        .version("1.0")
        .about("Bitcoin Transaction decoder")
        .arg(
            Arg::new("transaction_hex")
                .help("(string, required) Row Transaction hex")
                .required(true)
                .index(1),
        )
        .get_matches();

    // Retrieve transaction hex argument
    let hex_str = matches
        .get_one::<String>("transaction_hex")
        .expect("transaction_hex argument is required");

    // Call the decoder function from the library
    match decode_transaction(hex_str.clone()) {
        Ok(json_output) => println!("{json_output}"),
        Err(err) => {
            eprintln!("Error decoding transaction: {err}");
            std::process::exit(1);
        }
    }
}

// // https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2
