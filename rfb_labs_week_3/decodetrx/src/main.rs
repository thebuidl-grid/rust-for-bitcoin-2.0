use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    let matches = Command::new("Transaction decoder")
        .version("1.0")
        .about("Bitcoin Transaction decoder")
        .arg(
            Arg::new("transaction_hex")
                .required(true)
                .help("Raw Transaction hex string"),
        )
        .get_matches();

    let hex_str = matches
        .get_one::<String>("transaction_hex")
        .unwrap()
        .to_string();

    match decode_transaction(hex_str) {
        Ok(json_str) => {
            println!("{}", json_str);
        }
        Err(e) => {
            eprintln!("Error decoding transaction: {}", e);
            std::process::exit(1);
        }
    }
}
