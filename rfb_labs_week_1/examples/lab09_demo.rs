//! Manual runner for Lab 09 against a live Polar regtest node.
//!
//! Note: as in Lab 06, this node only populates `getrawtransaction`'s
//! `vin[].prevout` for *confirmed* transactions, so this demo mines one
//! confirming block between sending Alice's combined spend and decoding it,
//! calling the same underlying functions `audit_multi_utxo_spend` composes
//! (`send_combined_payment`, `decode_verbose_transaction`,
//! `identify_payment_and_change`, `calculate_fee`) rather than the composed
//! function itself, which decodes immediately after sending with no mining
//! step in between. `audit_multi_utxo_spend`'s end-to-end logic is verified
//! against mocks by `cargo test --test lab_09`.
//!
//! Usage: BITCOIN_CLI=/path/to/wrapper cargo run --example lab09_demo

use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use rfb_labs_week_1::labs::lab09_coin_selection::{
    confirmed_utxos_for_address, create_three_funding_transactions, send_combined_payment,
};
use rfb_labs_week_1::model::{MultiUtxoAudit, Utxo};
use rfb_labs_week_1::rpc::ProcessRpc;

const MINING_ADDRESS: &str = "bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv";

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    match create_wallet(&rpc, "alice") {
        Ok(()) => println!("createwallet(alice) -> ok"),
        Err(error) => println!("createwallet(alice) -> {error} (already exists is fine)"),
    }

    let alice_address =
        get_new_address(&rpc, "alice", "alice_funding").expect("getnewaddress(alice) failed");
    println!("alice address = {alice_address}");

    let funding_txids = create_three_funding_transactions(&rpc, "miner", &alice_address)
        .expect("create_three_funding_transactions failed");
    println!("funding txids = {funding_txids:?}");

    mine_blocks(&rpc, MINING_ADDRESS, 1).expect("confirming mine_blocks failed");
    println!("mined 1 block to confirm the three funding transactions");

    let funding_utxos = confirmed_utxos_for_address(&rpc, "alice", &alice_address)
        .expect("confirmed_utxos_for_address failed");
    println!("alice confirmed UTXOs ({}):", funding_utxos.len());
    for utxo in &funding_utxos {
        println!("  {utxo:?}");
    }

    let receiver_address =
        get_new_address(&rpc, "receiver", "alice_payment").expect("getnewaddress(receiver) failed");
    println!("new receiver address = {receiver_address}");

    let spend_txid = send_combined_payment(&rpc, "alice", &receiver_address)
        .expect("send_combined_payment failed");
    println!("spend txid = {spend_txid}");

    mine_blocks(&rpc, MINING_ADDRESS, 1).expect("confirming mine_blocks failed");
    println!("mined 1 block to confirm the combined spend");

    let transaction = decode_verbose_transaction(&rpc, &spend_txid).expect("decode failed");
    let payment_and_change =
        identify_payment_and_change(&transaction, &receiver_address).expect("identify failed");
    let fee = calculate_fee(&transaction).expect("fee calc failed");

    let audit = MultiUtxoAudit {
        funding_outpoints: funding_utxos.iter().map(Utxo::outpoint).collect(),
        spend_txid,
        spend_input_count: transaction.inputs.len(),
        payment_and_change,
        fee,
    };
    println!("{audit:#?}");
}
