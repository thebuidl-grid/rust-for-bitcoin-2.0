//! Lab 06 evidence runner.
//!
//! Usage: cargo run --example lab06 [-- <txid> <receiver_address>]
//!
//! Decodes the unconfirmed payment transaction and proves value conservation:
//! sum(inputs) = payment + change + fee.

use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints,
};
use rfb_labs_week_1::rpc::ProcessRpc;

const DEFAULT_TXID: &str = "a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b";
const DEFAULT_RECEIVER_ADDRESS: &str = "bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn";

fn main() {
    let mut args = std::env::args().skip(1);
    let txid = args.next().unwrap_or_else(|| DEFAULT_TXID.to_owned());
    let receiver_address = args
        .next()
        .unwrap_or_else(|| DEFAULT_RECEIVER_ADDRESS.to_owned());

    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let transaction = decode_verbose_transaction(&rpc, &txid).expect("decode transaction");

    println!("txid:  {}", transaction.txid);
    println!("vsize: {}", transaction.vsize);

    println!("inputs:");
    for input in &transaction.inputs {
        println!(
            "  {}:{} value={}",
            input.previous_output.txid, input.previous_output.vout, input.previous_value
        );
    }

    println!("outputs:");
    for output in &transaction.outputs {
        println!(
            "  vout={} value={} address={:?} script_pub_key_hex={}",
            output.vout, output.value, output.address, output.script_pub_key_hex
        );
    }

    let outpoints = input_outpoints(&transaction);
    println!(
        "consumed outpoints: {:?}",
        outpoints
            .iter()
            .map(|point| format!("{}:{}", point.txid, point.vout))
            .collect::<Vec<_>>()
    );

    let payment_and_change = identify_payment_and_change(&transaction, &receiver_address)
        .expect("identify payment and change");
    println!("payment output: {:?}", payment_and_change.payment);
    println!("change output:  {:?}", payment_and_change.change);

    let fee = calculate_fee(&transaction).expect("calculate fee");

    let input_total: f64 = transaction
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();
    let payment_value = payment_and_change.payment.value;
    let change_value = payment_and_change
        .change
        .as_ref()
        .map(|change| change.value)
        .unwrap_or(0.0);

    println!("fee: {fee}");
    println!(
        "sum(inputs) = {input_total}  ==  payment({payment_value}) + change({change_value}) + fee({fee}) = {}",
        payment_value + change_value + fee
    );
}
