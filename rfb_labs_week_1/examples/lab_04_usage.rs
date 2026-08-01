use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::{attempt_payment, mine_blocks};
use rfb_labs_week_1::labs::lab04_utxos::list_unspent;
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
    let result_attempt_payment =
        attempt_payment(&client, miner_wallet, &receiver_address, 1.0);
    println!("attempt_payment 1: ");
    println!("{result_attempt_payment:?}");

    mine_blocks(&client, &miner_address, 1)?;
    println!("1 block mined");

    let result_attempt_payment2 =
        attempt_payment(&client, miner_wallet, &receiver_address, 2.0);
    println!("attempt_payment 2: ");
    println!("{result_attempt_payment2:?}");

    mine_blocks(&client, &miner_address, 1)?;
    println!("1 block mined");

    let result_unspent = list_unspent(&client, receiver_wallet)?;
    println!("receiver unspent: {result_unspent:?}");

    Ok(())
    
}