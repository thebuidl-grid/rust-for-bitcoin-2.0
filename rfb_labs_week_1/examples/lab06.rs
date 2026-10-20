//! Runs the Lab 06 functions against a real Polar/regtest node.
//!
//! Decodes a specific, already-confirmed TXID from the Lab 05 payment.
//! `getrawtransaction` verbosity 2 only fills in `prevout` when block undo
//! data is available, i.e. once the transaction has been mined — an
//! unconfirmed (mempool-only) transaction won't have it yet.
//!
//! Run with: cargo run --example lab06

use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints,
};
use rfb_labs_week_1::rpc::ProcessRpc;

const TXID: &str = "f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef";
const RECEIVER_ADDRESS: &str = "bcrt1q8xnsl28ymp70jxzmf7gnxx59aaa4tk6nl0cxs2";

fn main() {
    let client = ProcessRpc::new("docker").with_base_args([
        "exec",
        "-u",
        "bitcoin",
        "polar-n1-backend1",
        "bitcoin-cli",
        "-regtest",
    ]);

    let transaction = decode_verbose_transaction(&client, TXID).expect("decode failed");
    println!("{transaction:#?}");

    println!("consumed outpoints: {:#?}", input_outpoints(&transaction));

    let payment_and_change = identify_payment_and_change(&transaction, RECEIVER_ADDRESS)
        .expect("identify_payment_and_change failed");
    println!("{payment_and_change:#?}");

    let fee = calculate_fee(&transaction).expect("calculate_fee failed");
    println!("fee: {fee}");
}
