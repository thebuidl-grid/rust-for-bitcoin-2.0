use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    let matches = Command::new("transaction-decoder")
        .version("1.0")
        .about("Decode a raw Bitcoin transaction")
        .arg(
            Arg::new("transaction_hex")
                .help("Raw Bitcoin transaction in hexadecimal")
                .required(true)
                .index(1),
        )
        .get_matches();

    let transaction_hex = matches
        .get_one::<String>("transaction_hex")
        .expect("transaction_hex is required");

    match decode_transaction(transaction_hex.clone()) {
        Ok(transaction) => println!("{}", transaction),
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    }
}