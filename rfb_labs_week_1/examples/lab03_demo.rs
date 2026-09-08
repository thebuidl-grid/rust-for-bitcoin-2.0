//! Manual runner for Lab 03 against a live Polar regtest node.
//! Usage: BITCOIN_CLI=/path/to/wrapper cargo run --example lab03_demo

use rfb_labs_week_1::labs::lab03_maturity::demonstrate_coinbase_maturity;
use rfb_labs_week_1::rpc::ProcessRpc;

const MINING_ADDRESS: &str = "bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv";
const CLASSMATE_ADDRESS: &str = "bcrt1qxmst06mxnlgm5u7tscqsvvf892x8ulsasrl5ua";

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    let report = demonstrate_coinbase_maturity(&rpc, "miner", MINING_ADDRESS, CLASSMATE_ADDRESS)
        .expect("demonstrate_coinbase_maturity failed");

    println!("{report:#?}");
}
