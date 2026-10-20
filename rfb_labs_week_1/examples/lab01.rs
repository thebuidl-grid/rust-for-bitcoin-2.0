//! Lab 01 evidence runner.
//!
//! Usage: cargo run --example lab01

use rfb_labs_week_1::labs::lab01_network::inspect_network;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    match inspect_network(&rpc) {
        Ok(snapshot) => {
            println!("node is running: yes (RPC call succeeded)");
            println!("chain:            {}", snapshot.chain);
            println!("block_height:     {}", snapshot.block_height);
            println!("best_block_hash:  {}", snapshot.best_block_hash);
        }
        Err(error) => {
            eprintln!("lab01 failed: {error}");
            std::process::exit(1);
        }
    }
}
