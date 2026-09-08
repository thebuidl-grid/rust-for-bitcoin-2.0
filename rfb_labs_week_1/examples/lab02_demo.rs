//! Manual runner for Lab 02 against a live Polar regtest node.
//! Usage: BITCOIN_CLI=/path/to/wrapper cargo run --example lab02_demo

use rfb_labs_week_1::labs::lab02_wallets::{
    address_belongs_to_wallet, create_wallet, get_new_address, list_wallets,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    for wallet in ["miner", "receiver"] {
        match create_wallet(&rpc, wallet) {
            Ok(()) => println!("createwallet({wallet}) -> ok"),
            Err(error) => println!("createwallet({wallet}) -> {error} (already exists is fine)"),
        }
    }

    let wallets = list_wallets(&rpc).expect("listwallets failed");
    println!("loaded wallets = {wallets:?}");

    let mining_address =
        get_new_address(&rpc, "miner", "mining").expect("getnewaddress(miner) failed");
    let classmate_address =
        get_new_address(&rpc, "receiver", "classmate").expect("getnewaddress(receiver) failed");

    println!("mining address    = {mining_address}");
    println!("classmate address = {classmate_address}");

    let miner_owns_mining =
        address_belongs_to_wallet(&rpc, "miner", &mining_address).expect("getaddressinfo failed");
    let receiver_owns_classmate = address_belongs_to_wallet(&rpc, "receiver", &classmate_address)
        .expect("getaddressinfo failed");
    let miner_owns_classmate = address_belongs_to_wallet(&rpc, "miner", &classmate_address)
        .expect("getaddressinfo failed");

    println!("miner owns mining address?       = {miner_owns_mining}");
    println!("receiver owns classmate address?  = {receiver_owns_classmate}");
    println!("miner owns classmate address?     = {miner_owns_classmate} (expected false)");
}
