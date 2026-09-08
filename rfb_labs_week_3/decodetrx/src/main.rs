use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    let matches = Command::new("decodetrx")
        .version("1.0")
        .about("Decode a raw Bitcoin transaction and print it as JSON")
        .arg(
            Arg::new("hex")
                .help("Raw transaction hex string")
                .required(true)
                .index(1),
        )
        .get_matches();

    let hex: &String = matches.get_one("hex").expect("hex argument is required");

    match decode_transaction(hex.clone()) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

// Test transactions:
// SegWit (mempool.space testnet):
// https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2
