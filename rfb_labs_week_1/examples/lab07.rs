//! Lab 07 evidence runner.
//!
//! Usage: cargo run --example lab07 [-- <txid> <miner_address>]
//!
//! Mines a block, proves the txid left the mempool, reads confirmations from
//! the receiver's wallet view, and proves the confirming block's tx list
//! actually contains this txid.

use rfb_labs_week_1::labs::lab07_confirm::confirm_and_locate_transaction;
use rfb_labs_week_1::rpc::ProcessRpc;

const DEFAULT_TXID: &str = "a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b";
const DEFAULT_MINER_ADDRESS: &str = "bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn";

fn main() {
    let mut args = std::env::args().skip(1);
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

    let report = confirm_and_locate_transaction(&rpc, "receiver", &txid, &miner_address)
        .expect("confirm and locate transaction");

    println!("txid:                    {}", report.txid);
    println!("block_hash:              {}", report.block_hash);
    println!("confirmations:           {}", report.confirmations);
    println!("mempool_is_empty:        {}", report.mempool_is_empty);
    println!(
        "transaction_is_in_block: {}",
        report.transaction_is_in_block
    );
}
