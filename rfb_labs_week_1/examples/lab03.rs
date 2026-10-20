//! Runs the Lab 03 functions against a real Polar/regtest node.
//!
//! Uses a fresh `miner2` wallet with no other funds, so the premature-spend
//! check inside `demonstrate_coinbase_maturity` behaves as intended: a
//! wallet that already holds other spendable coins (like `miner`, after the
//! manual `bitcoin-cli` walkthrough) would let the "premature" 1 BTC payment
//! succeed from its other funds, defeating the point of the proof.
//!
//! Run with: cargo run --example lab03

use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::demonstrate_coinbase_maturity;
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

    create_wallet(&client, "miner2").expect("create_wallet failed");
    let miner2_address =
        get_new_address(&client, "miner2", "maturity-demo").expect("get_new_address failed");
    let receiver_address = get_new_address(&client, "receiver", "maturity-demo-receiver")
        .expect("get_new_address failed");

    let report =
        demonstrate_coinbase_maturity(&client, "miner2", &miner2_address, &receiver_address)
            .expect("demonstrate_coinbase_maturity failed");

    println!("{report:#?}");
}
