use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    let matches = Command::new("Transaction Decoder")
        .version("1.0")
        .author("Bitcoin Transaction Decoder")
        .about("Decodes Bitcoin transactions from hex format")
        .arg(
            Arg::new("transaction_hex")
                .short('t')
                .long("tx")
                .value_name("HEX")
                .help("Transaction in hexadecimal format")
                .required(true)
        )
        .get_matches();
    
    let transaction_hex = matches
        .get_one::<String>("transaction_hex")
        .expect("Transaction hex is required");
    
    match decode_transaction(transaction_hex.clone()) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("Error decoding transaction: {}", e);
            std::process::exit(1);
        }
    }
}
