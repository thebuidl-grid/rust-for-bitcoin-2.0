//! Manual runner for Lab 01 against a live Polar regtest node.
//!
//! Usage: BITCOIN_CLI=/path/to/bitcoin-cli-wrapper cargo run --example lab01_demo

use rfb_labs_week_1::labs::lab01_network::inspect_network;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    match inspect_network(&rpc) {
        Ok(snapshot) => {
            println!("chain            = {}", snapshot.chain);
            println!("block_height     = {}", snapshot.block_height);
            println!("best_block_hash  = {}", snapshot.best_block_hash);
        }
        Err(error) => eprintln!("inspect_network failed: {error}"),
    }
}
