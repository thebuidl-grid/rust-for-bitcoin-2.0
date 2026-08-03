use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let rpc = ProcessRpc::new("docker").with_base_args([
        "exec",
        "polar-n3-backend1",
        "bitcoin-cli",
        "-regtest",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let txid = "a9d0febd729cf46b33a44e7a2007266ac1332b554cfd6f98aae864036701aaa9";
    let classmate_address = "bcrt1q0uwx0lqm5p8geyd0njz0hjmcl5fnrdl6ka56at";

    let transaction = decode_verbose_transaction(&rpc, txid).unwrap();
    println!("decoded transaction: {transaction:#?}");

    let outpoints = input_outpoints(&transaction);
    println!("consumed outpoints: {outpoints:#?}");

    let payment_and_change = identify_payment_and_change(&transaction, classmate_address).unwrap();
    println!("payment and change: {payment_and_change:#?}");

    let fee = calculate_fee(&transaction).unwrap();
    println!("fee: {fee}");

    let input_total: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let output_total: f64 = transaction.outputs.iter().map(|o| o.value).sum();
    println!(
        "value conservation: {input_total} = {output_total} + {fee} (sum(inputs) = sum(outputs) + fee)"
    );
}
