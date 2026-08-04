//! Manual runner for Lab 05 against a live Polar regtest node.
//! Usage: BITCOIN_CLI=/path/to/wrapper cargo run --example lab05_demo

use rfb_labs_week_1::labs::lab05_mempool::observe_unconfirmed_payment;
use rfb_labs_week_1::rpc::ProcessRpc;

const CLASSMATE_ADDRESS: &str = "bcrt1qxmst06mxnlgm5u7tscqsvvf892x8ulsasrl5ua";

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    let observation =
        observe_unconfirmed_payment(&rpc, "miner", "receiver", CLASSMATE_ADDRESS, 1.0)
            .expect("observe_unconfirmed_payment failed");

    println!("{observation:#?}");
}
