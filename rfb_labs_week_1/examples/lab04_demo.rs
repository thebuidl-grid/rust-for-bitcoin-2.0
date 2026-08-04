//! Manual runner for Lab 04 against a live Polar regtest node.
//! Usage: BITCOIN_CLI=/path/to/wrapper cargo run --example lab04_demo

use rfb_labs_week_1::labs::lab04_utxos::{
    list_unspent, outpoint, select_spendable_utxo, sum_spendable_utxos,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    let utxos = list_unspent(&rpc, "miner").expect("listunspent failed");
    println!("miner UTXOs ({} total):", utxos.len());
    for utxo in &utxos {
        println!("  {utxo:?}");
    }

    let chosen = select_spendable_utxo(&utxos).expect("no spendable UTXO found");
    println!("\nselected spendable UTXO = {chosen:?}");
    println!("its outpoint            = {:?}", outpoint(&chosen));

    let total = sum_spendable_utxos(&utxos);
    println!("sum of spendable UTXOs   = {total}");
}
