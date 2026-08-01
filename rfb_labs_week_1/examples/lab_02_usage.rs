use rfb_labs_week_1::labs::lab02_wallets::{address_belongs_to_wallet, create_wallet, get_new_address, list_wallets};
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

    let receiver_address = get_new_address(&client, receiver_wallet, "classmate")?;
    println!("receiver_address: {}", receiver_address);

    let loaded_wallets = list_wallets(&client)?;
    println!("loaded_wallets: {:?}", loaded_wallets);

    let address_belongs_to_wallet_miner =
        address_belongs_to_wallet(&client, miner_wallet, &miner_address)?;
    println!(
        "address_belongs_to_wallet_miner: {:?}",
        address_belongs_to_wallet_miner
    );

    let address_belongs_to_wallet_receiver =
        address_belongs_to_wallet(&client, receiver_wallet, &receiver_address)?;
    println!(
        "address_belongs_to_wallet_receiver: {:?}",
        address_belongs_to_wallet_receiver
    );
    Ok(())

    
}