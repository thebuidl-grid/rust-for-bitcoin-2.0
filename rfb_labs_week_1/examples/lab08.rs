//! Runs the Lab 08 functions against a real Polar/regtest node.
//!
//! Sends a fresh payment, confirms it once, then proves 1 confirmation
//! becomes 6 after mining exactly 5 more blocks (all mining here is done
//! programmatically by `mine_additional_blocks`, so the block count is
//! exact — no terminal paste/double-send risk).
//!
//! Run with: cargo run --example lab08

use rfb_labs_week_1::labs::lab02_wallets::get_new_address;
use rfb_labs_week_1::labs::lab05_mempool::send_btc;
use rfb_labs_week_1::labs::lab07_confirm::mine_one_block;
use rfb_labs_week_1::labs::lab08_security::build_security_report;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let client = ProcessRpc::new("docker").with_base_args([
        "exec",
        "-u",
        "bitcoin",
        "polar-n1-backend1",
        "bitcoin-cli",
        "-regtest",
    ]);

    let receiver_address =
        get_new_address(&client, "receiver", "security-demo").expect("get_new_address failed");
    let txid = send_btc(&client, "miner", &receiver_address, 1.0).expect("send_btc failed");
    println!("sent txid: {txid}");

    let miner_address =
        get_new_address(&client, "miner", "security-demo-block").expect("get_new_address failed");
    let block_hash = mine_one_block(&client, &miner_address).expect("mine_one_block failed");
    println!("confirmed in block: {block_hash}");

    let report = build_security_report(&client, "miner", &txid, &block_hash, &miner_address)
        .expect("build_security_report failed");

    println!("{report:#?}");
}
