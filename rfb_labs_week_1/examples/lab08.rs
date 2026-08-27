//! Lab 08 evidence runner.
//!
//! Usage: cargo run --example lab08 -- <block_hash> [<txid> <miner_address>]
//!
//! <block_hash> is the confirming block's hash printed by `cargo run --example
//! lab07` (its `block_hash:` line) — there's no safe default for it since it's
//! unique to your run.

use rfb_labs_week_1::labs::lab08_security::build_security_report;
use rfb_labs_week_1::rpc::ProcessRpc;

const DEFAULT_TXID: &str = "a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b";
const DEFAULT_MINER_ADDRESS: &str = "bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn";

fn main() {
    let mut args = std::env::args().skip(1);
    let block_hash = args
        .next()
        .expect("usage: cargo run --example lab08 -- <block_hash> [<txid> <miner_address>]");
    let txid = args.next().unwrap_or_else(|| DEFAULT_TXID.to_owned());
    let miner_address = args
        .next()
        .unwrap_or_else(|| DEFAULT_MINER_ADDRESS.to_owned());

    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let report = build_security_report(&rpc, "receiver", &txid, &block_hash, &miner_address)
        .expect("build security report");

    println!("header:");
    println!("  hash:                {}", report.header.hash);
    println!("  height:              {}", report.header.height);
    println!(
        "  previous_block_hash: {:?}",
        report.header.previous_block_hash
    );
    println!("  merkle_root:         {}", report.header.merkle_root);
    println!("  nonce:               {}", report.header.nonce);
    println!("  difficulty:          {}", report.header.difficulty);
    println!("  bits:                {}", report.header.bits);
    println!("  confirmations:       {}", report.header.confirmations);
    println!("  chainwork:           {}", report.header.chainwork);
    println!("confirmations_before:  {}", report.confirmations_before);
    println!("confirmations_after:   {}", report.confirmations_after);
}
