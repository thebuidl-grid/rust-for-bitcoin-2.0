//! Runs the Lab 09 functions against a real Polar/regtest node.
//!
//! Funds Alice with three fresh 0.4 BTC UTXOs, mines them, then has her send
//! 1 BTC (more than any single UTXO covers).
//!
//! Note: calling `audit_multi_utxo_spend` directly here fails with
//! `MissingField("prevout")` — it sends and decodes back-to-back (matching
//! its test), but `getrawtransaction` verbosity 2 only fills in `prevout`
//! once block undo data exists, i.e. once the transaction is confirmed (the
//! same real-node limitation documented in Lab 06). So below, the same
//! underlying steps `audit_multi_utxo_spend` performs are run manually with
//! a mining step inserted in between, to demonstrate the full audit
//! succeeding against the live node.
//!
//! Run with: cargo run --example lab09

use rfb_labs_week_1::labs::lab02_wallets::get_new_address;
use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use rfb_labs_week_1::labs::lab07_confirm::mine_one_block;
use rfb_labs_week_1::labs::lab09_coin_selection::{
    confirmed_utxos_for_address, create_three_funding_transactions, send_combined_payment,
};
use rfb_labs_week_1::model::MultiUtxoAudit;
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

    let alice_address = get_new_address(&client, "alice", "coin-selection-demo2")
        .expect("get_new_address (alice) failed");

    let funding_txids = create_three_funding_transactions(&client, "miner", &alice_address)
        .expect("create_three_funding_transactions failed");
    println!("funding txids: {funding_txids:#?}");

    let miner_address = get_new_address(&client, "miner", "coin-selection-demo2-block")
        .expect("get_new_address failed");
    mine_one_block(&client, &miner_address).expect("mine_one_block failed");

    let funding_utxos = confirmed_utxos_for_address(&client, "alice", &alice_address)
        .expect("confirmed_utxos_for_address failed");
    println!("Alice's confirmed UTXOs: {}", funding_utxos.len());

    let receiver_address = get_new_address(&client, "receiver", "coin-selection-demo2-receiver")
        .expect("get_new_address (receiver) failed");

    let spend_txid = send_combined_payment(&client, "alice", &receiver_address)
        .expect("send_combined_payment failed");
    mine_one_block(&client, &miner_address).expect("mine_one_block (confirm spend) failed");

    let transaction = decode_verbose_transaction(&client, &spend_txid).expect("decode failed");
    let payment_and_change = identify_payment_and_change(&transaction, &receiver_address)
        .expect("identify_payment_and_change failed");
    let fee = calculate_fee(&transaction).expect("calculate_fee failed");

    let audit = MultiUtxoAudit {
        funding_outpoints: funding_utxos.iter().map(|u| u.outpoint()).collect(),
        spend_txid,
        spend_input_count: transaction.inputs.len(),
        payment_and_change,
        fee,
    };

    println!("{audit:#?}");
}
