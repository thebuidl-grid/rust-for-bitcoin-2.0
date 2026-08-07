//! Lab 09 evidence runner.
//!
//! Usage: cargo run --example lab09 [-- <miner_address>]
//!
//! Creates an alice wallet, funds it with three separate 0.4 BTC payments,
//! confirms them, proves three distinct UTXOs, then has Alice spend 1 BTC
//! (requiring multiple inputs) to a fresh receiver address and audits the
//! spend. The combined spend is confirmed before decoding, for the same
//! reason Lab 06 needed to: `getrawtransaction ... 2` only returns `prevout`
//! once block undo data exists, which requires confirmation.

use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use rfb_labs_week_1::labs::lab09_coin_selection::{
    confirmed_utxos_for_address, create_three_funding_transactions, send_combined_payment,
};
use rfb_labs_week_1::model::Utxo;
use rfb_labs_week_1::rpc::ProcessRpc;

const DEFAULT_MINER_ADDRESS: &str = "bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn";

fn main() {
    let miner_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_MINER_ADDRESS.to_owned());

    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    create_wallet(&rpc, "alice").expect("create alice wallet");
    let alice_address = get_new_address(&rpc, "alice", "alice-funding").expect("alice address");
    println!("alice address: {alice_address}");

    let funding_txids =
        create_three_funding_transactions(&rpc, "miner", &alice_address).expect("fund alice");
    println!("funding txids: {funding_txids:?}");

    mine_blocks(&rpc, &miner_address, 1).expect("confirm funding transactions");

    let funding_utxos =
        confirmed_utxos_for_address(&rpc, "alice", &alice_address).expect("alice's UTXOs");
    println!("alice confirmed UTXO count: {}", funding_utxos.len());
    for utxo in &funding_utxos {
        println!("  {}:{} amount={}", utxo.txid, utxo.vout, utxo.amount);
    }

    let receiver_address =
        get_new_address(&rpc, "receiver", "alice-payment").expect("new receiver address");
    println!("new receiver address: {receiver_address}");

    let spend_txid =
        send_combined_payment(&rpc, "alice", &receiver_address).expect("send combined payment");
    println!("spend txid: {spend_txid}");

    mine_blocks(&rpc, &miner_address, 1).expect("confirm combined spend");

    let decoded = decode_verbose_transaction(&rpc, &spend_txid).expect("decode combined spend");
    let payment_and_change = identify_payment_and_change(&decoded, &receiver_address)
        .expect("identify payment and change");
    let fee = calculate_fee(&decoded).expect("calculate fee");

    println!("spend input count: {}", decoded.inputs.len());
    println!("payment output: {:?}", payment_and_change.payment);
    println!("change output:  {:?}", payment_and_change.change);
    println!("fee: {fee}");
    println!(
        "funding outpoints: {:?}",
        funding_utxos
            .iter()
            .map(Utxo::outpoint)
            .map(|point| format!("{}:{}", point.txid, point.vout))
            .collect::<Vec<_>>()
    );
}
