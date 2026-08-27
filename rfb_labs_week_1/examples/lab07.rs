//! Runs the Lab 07 functions against a real Polar/regtest node.
//!
//! Run with: cargo run --example lab07

use rfb_labs_week_1::labs::lab02_wallets::get_new_address;
use rfb_labs_week_1::labs::lab05_mempool::send_btc;
use rfb_labs_week_1::labs::lab07_confirm::confirm_and_locate_transaction;
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
        get_new_address(&client, "receiver", "confirm-demo").expect("get_new_address failed");
    let txid = send_btc(&client, "miner", &receiver_address, 1.0).expect("send_btc failed");
    println!("sent txid: {txid}");

    let miner_address =
        get_new_address(&client, "miner", "confirm-demo-block").expect("get_new_address failed");

    let report = confirm_and_locate_transaction(&client, "miner", &txid, &miner_address)
        .expect("confirm_and_locate_transaction failed");

    println!("{report:#?}");
}
