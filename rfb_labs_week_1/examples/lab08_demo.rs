//! Manual runner for Lab 08 against a live Polar regtest node.
//! Usage: BITCOIN_CLI=/path/to/wrapper cargo run --example lab08_demo

use rfb_labs_week_1::labs::lab08_security::build_security_report;
use rfb_labs_week_1::rpc::ProcessRpc;

const TXID: &str = "cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe";
const BLOCK_HASH: &str = "0e0e0b599c631219e78abae3a7c965c07117cb5943e90cc6bdf72df803c38c58";
const MINING_ADDRESS: &str = "bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv";

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    let report = build_security_report(&rpc, "receiver", TXID, BLOCK_HASH, MINING_ADDRESS)
        .expect("build_security_report failed");

    println!("{report:#?}");
}
