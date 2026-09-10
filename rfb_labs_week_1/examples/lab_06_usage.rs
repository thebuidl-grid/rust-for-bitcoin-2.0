use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::{attempt_payment, mine_blocks};
use rfb_labs_week_1::labs::lab05_mempool::{get_raw_mempool, get_transaction_status, observe_unconfirmed_payment, send_btc};
use rfb_labs_week_1::labs::lab06_decode::{calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ProcessRpc::new("docker").with_base_args([
        "exec",
        "polar-n1-backend1",
        "bitcoin-cli",
        "-regtest",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);
 

  let miner_wallet = "miner";
    let receiver_wallet = "receiver";
    let _ = create_wallet(&client, miner_wallet);
    let _ = create_wallet(&client, receiver_wallet);
    println!("miner_wallet: {}", miner_wallet);
    println!("receiver_wallet: {}", receiver_wallet);


let miner_address = get_new_address(&client, miner_wallet, "mining")?;
    println!("miner_address: {}", miner_address);
    mine_blocks(&client, &miner_address, 101)?;
    println!("101 blocks mined");

    let receiver_address = get_new_address(&client, receiver_wallet, "classmate")?;
    println!("receiver_address: {}", receiver_address);
    let txid =
        send_btc(&client, miner_wallet, &receiver_address, 1.0)?;
    println!("payment txid: ");
    println!("{txid:#?}");


    let transaction = decode_verbose_transaction(&client, &txid)?;
    println!("decoded transaction: {transaction:#?}");

    let outpoints = input_outpoints(&transaction);
    println!("consumed outpoints: {outpoints:#?}");

    let payment_and_change = identify_payment_and_change(&transaction, &receiver_address)?;
    println!("payment and change: {payment_and_change:#?}");

    let fee = calculate_fee(&transaction)?;
    println!("fee: {fee}");

    let input_total: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let output_total: f64 = transaction.outputs.iter().map(|o| o.value).sum();
 
    Ok(())
    
}