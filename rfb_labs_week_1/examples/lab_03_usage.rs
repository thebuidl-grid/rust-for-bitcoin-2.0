use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::{attempt_payment, demonstrate_coinbase_maturity, get_balances};
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
    // let result_mine_blocks = mine_blocks(&client, &miner_address, 1);
    println!("miner_address: {}", miner_address);
    println!("mine_blocks: ");
    // println!("{result_mine_blocks:?}");

    let result_get_balances = get_balances(&client, miner_wallet);
    println!("get_balances: ");
    println!("{result_get_balances:?}");

    let receiver_address = get_new_address(&client, receiver_wallet, "classmate")?;
    let result_attempt_payment =
        attempt_payment(&client, miner_wallet, &receiver_address, 1.0);
    println!("receiver_address: {}", receiver_address);
    println!("attempt_payment: ");
    println!("{result_attempt_payment:?}");

    let result_coinbase = demonstrate_coinbase_maturity(
        &client,
        miner_wallet,
        &miner_address,
        &receiver_address,
    )?;
    println!("result_coinbase: ");
    println!("{result_coinbase:?}");

    let result_attempt_payment =
        attempt_payment(&client, miner_wallet, &receiver_address, 1.0);
    println!("receiver_address: {}", receiver_address);
    println!("attempt_payment: ");
    println!("{result_attempt_payment:?}");

    Ok(())

    
}