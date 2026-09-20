use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    let matches = Command::new("Transaction decoder")
        .version("1.0")
        .about("Bitcoin Transaction decoder")
        .arg(
            Arg::new("transaction_hex")
                .help("(string, required) Raw transaction hex")
                .required(true)
                .index(1),
        )
        .get_matches();

    let transaction_hex = matches
        .get_one::<String>("transaction_hex")
        .expect("transaction_hex is required")
        .to_string();

    match decode_transaction(transaction_hex) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("Error decoding transaction: {e}"),
    }
}
