//! Runs the Lab 05 functions against a real Polar/regtest node.
//!
//! Sends a fresh 1 BTC payment from `miner` to `receiver` and observes it
//! while still unconfirmed. Do not mine any blocks before or during this run.
//!
//! Run with: cargo run --example lab05

use rfb_labs_week_1::labs::lab02_wallets::get_new_address;
use rfb_labs_week_1::labs::lab05_mempool::observe_unconfirmed_payment;
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
        get_new_address(&client, "receiver", "mempool-demo").expect("get_new_address failed");

    let observation =
        observe_unconfirmed_payment(&client, "miner", "receiver", &receiver_address, 1.0)
            .expect("observe_unconfirmed_payment failed");

    println!("{observation:#?}");
}
