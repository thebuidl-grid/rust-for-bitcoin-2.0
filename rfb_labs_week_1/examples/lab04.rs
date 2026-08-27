//! Lab 04 evidence runner.
//!
//! Usage: cargo run --example lab04
//!
//! Lists the miner wallet's UTXOs, records one spendable UTXO's fields and
//! outpoint, then independently sums all spendable UTXOs and reconciles that
//! sum against Bitcoin Core's own trusted wallet balance.

use rfb_labs_week_1::labs::lab03_maturity::get_balances;
use rfb_labs_week_1::labs::lab04_utxos::{
    list_unspent, outpoint, select_spendable_utxo, sum_spendable_utxos,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let utxos = list_unspent(&rpc, "miner").expect("list unspent");
    println!("miner UTXO count: {}", utxos.len());

    let selected = select_spendable_utxo(&utxos).expect("at least one spendable UTXO");
    println!("selected UTXO:");
    println!("  txid:            {}", selected.txid);
    println!("  vout:            {}", selected.vout);
    println!("  amount:          {}", selected.amount);
    println!("  confirmations:   {}", selected.confirmations);
    println!("  address:         {:?}", selected.address);
    println!("  script_pub_key:  {}", selected.script_pub_key);
    println!("  spendable:       {}", selected.spendable);

    let selected_outpoint = outpoint(&selected);
    println!(
        "selected outpoint: {}:{}",
        selected_outpoint.txid, selected_outpoint.vout
    );

    let independent_sum = sum_spendable_utxos(&utxos);
    let wallet_trusted_balance = get_balances(&rpc, "miner").expect("get balances").trusted;

    println!("independent sum of spendable UTXOs: {independent_sum}");
    println!("wallet trusted balance (getbalances): {wallet_trusted_balance}");
    println!(
        "reconciled: {}",
        (independent_sum - wallet_trusted_balance).abs() < 0.000_000_01
    );
}
