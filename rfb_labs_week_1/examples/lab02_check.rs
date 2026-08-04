use rfb_labs_week_1::labs::lab02_wallets::{
    address_belongs_to_wallet, create_wallet, get_new_address, list_wallets,
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

    create_wallet(&rpc, "miner").unwrap();
    create_wallet(&rpc, "receiver").unwrap();

    let wallets = list_wallets(&rpc).unwrap();
    println!("loaded wallets: {wallets:?}");

    let mining_address = get_new_address(&rpc, "miner", "mining").unwrap();
    println!("mining address: {mining_address}");

    let classmate_address = get_new_address(&rpc, "receiver", "classmate").unwrap();
    println!("classmate address: {classmate_address}");

    let mining_is_miners = address_belongs_to_wallet(&rpc, "miner", &mining_address).unwrap();
    let classmate_is_receivers =
        address_belongs_to_wallet(&rpc, "receiver", &classmate_address).unwrap();

    println!("mining address belongs to miner: {mining_is_miners}");
    println!("classmate address belongs to receiver: {classmate_is_receivers}");
}
