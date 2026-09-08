use clap::{Arg, Command, Parser};
use decodetrx::{decode_transaction, CLI};

fn main() {
    // Define CLI using Clap
    let cli = CLI::parse();
   
    // Retrieve transaction hex argument
    let hex = cli.transaction_hex;
    

    // Call the decoder function from the library
    let result = decode_transaction(hex).unwrap();
    println!("{}", result);
    
}

// // https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2