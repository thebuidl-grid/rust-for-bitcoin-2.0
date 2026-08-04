use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::{mine_blocks};
use rfb_labs_week_1::labs::lab05_mempool::{get_raw_mempool, get_transaction_status, observe_unconfirmed_payment, send_btc};
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

    let mempool = get_raw_mempool(&client);
        println!("mempool: {:?}", mempool);

   

    let transaction_status = get_transaction_status(&client, receiver_wallet, &txid)?;
        println!("transaction_status: {:?}", transaction_status);

    let observation =
        observe_unconfirmed_payment(&client, "miner", "receiver", &receiver_address, 1.0)?;

    println!("Balance: ");
    println!("{:#?}", observation.receiver_balance);

    Ok(())
    
}