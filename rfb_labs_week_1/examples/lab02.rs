//! Lab 02 evidence runner.
//!
//! Usage: cargo run --example lab02
//!
//! Assumes a fresh node with no `miner`/`receiver` wallets yet. If you rerun
//! this against a node that already has them, `create_wallet` will fail with
//! "already exists" — that's expected, not a bug.

use rfb_labs_week_1::labs::lab02_wallets::{
    address_belongs_to_wallet, create_wallet, get_new_address, list_wallets,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18444",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    create_wallet(&rpc, "miner").expect("create miner wallet");
    create_wallet(&rpc, "receiver").expect("create receiver wallet");

    let wallets = list_wallets(&rpc).expect("list wallets");
    println!("loaded wallets: {wallets:?}");

    let mining_address = get_new_address(&rpc, "miner", "mining").expect("miner address");
    let classmate_address =
        get_new_address(&rpc, "receiver", "classmate").expect("receiver address");
    println!("miner address (mining):       {mining_address}");
    println!("receiver address (classmate): {classmate_address}");

    let miner_owns_it =
        address_belongs_to_wallet(&rpc, "miner", &mining_address).expect("check miner ownership");
    let receiver_owns_it = address_belongs_to_wallet(&rpc, "receiver", &classmate_address)
        .expect("check receiver ownership");
    println!("miner wallet owns mining address:       {miner_owns_it}");
    println!("receiver wallet owns classmate address: {receiver_owns_it}");
}
