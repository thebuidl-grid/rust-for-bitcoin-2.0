//! Runs the Lab 04 functions against a real Polar/regtest node.
//!
//! Run with: cargo run --example lab04

use rfb_labs_week_1::labs::lab04_utxos::{
    list_unspent, outpoint, select_spendable_utxo, sum_spendable_utxos,
};
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

    let utxos = list_unspent(&client, "miner").expect("list_unspent failed");
    println!("miner wallet UTXO count: {}", utxos.len());

    let chosen = select_spendable_utxo(&utxos).expect("no spendable UTXO found");
    println!("selected UTXO: {chosen:#?}");
    println!("its outpoint: {:#?}", outpoint(&chosen));

    let total = sum_spendable_utxos(&utxos);
    println!("independently summed spendable balance: {total}");
}
