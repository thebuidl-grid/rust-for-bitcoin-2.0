//! Manual runner for Lab 06 against a live Polar regtest node.
//! Usage: BITCOIN_CLI=/path/to/wrapper cargo run --example lab06_demo

use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints,
};
use rfb_labs_week_1::rpc::ProcessRpc;

const TXID: &str = "cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe";
const CLASSMATE_ADDRESS: &str = "bcrt1qxmst06mxnlgm5u7tscqsvvf892x8ulsasrl5ua";

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    let transaction = decode_verbose_transaction(&rpc, TXID).expect("decode failed");
    println!("{transaction:#?}");

    let outpoints = input_outpoints(&transaction);
    println!("\nconsumed outpoints = {outpoints:?}");

    let payment_and_change =
        identify_payment_and_change(&transaction, CLASSMATE_ADDRESS).expect("identify failed");
    println!("\npayment_and_change = {payment_and_change:#?}");

    let fee = calculate_fee(&transaction).expect("fee calc failed");
    let input_sum: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let output_sum: f64 = transaction.outputs.iter().map(|o| o.value).sum();

    println!("\nsum(inputs)  = {input_sum}");
    println!("sum(outputs) = {output_sum}");
    println!("fee          = {fee}");
    println!(
        "sum(inputs) == sum(outputs) + fee ? {}",
        (input_sum - (output_sum + fee)).abs() < 1e-9
    );
}
