use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    // Define CLI using Clap
    let matches = Command::new("decodetrx")
        .version("1.0")
        .author("Your Name")
        .about("Decodes a raw Bitcoin transaction hex into human-readable JSON")
        .arg(
            Arg::new("hex")
                .help("The raw Bitcoin transaction hex string to decode")
                .required(true)
                .index(1),
        )
        .get_matches();
   
    // Retrieve transaction hex argument
    let hex_str = matches
        .get_one::<String>("hex")
        .expect("Transaction hex is required");
    

    // Call the decoder function from the library
    match decode_transaction(hex_str.to_string()) {
        Ok(decoded_tx) => {
            // Print the decoded struct nicely formatted as JSON
            match serde_json::to_string_pretty(&decoded_tx) {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("Failed to serialize transaction: {}", e),
            }
        }
        Err(e) => eprintln!("Error decoding transaction: {}", e),
    }
    
}

// // https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2