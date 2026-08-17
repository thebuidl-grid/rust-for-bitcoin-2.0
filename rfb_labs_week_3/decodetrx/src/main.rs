use clap::{Arg, Command};
use decodetrx::decode_transaction;

// Example transaction:
// https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2

fn main() {
    let matches = Command::new("decodetrx")
        .version("1.0")
        .about("Bitcoin transaction decoder")
        .arg(
            Arg::new("transaction_hex")
                .required(true)
                .help("(string, required) Raw transaction hex"),
        )
        .get_matches();

    // `transaction_hex` is declared required, so Clap exits before this point
    // if it is missing.
    let transaction_hex = matches
        .get_one::<String>("transaction_hex")
        .expect("transaction_hex is required");

    match decode_transaction(transaction_hex.clone()) {
        Ok(json) => println!("{}", json),
        Err(error) => {
            eprintln!("error: could not decode transaction: {}", error);
            std::process::exit(1);
        }
    }
}
