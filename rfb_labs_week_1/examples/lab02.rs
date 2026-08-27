//! Runs the Lab 02 functions against a real Polar/regtest node.
//!
//! Run with: cargo run --example lab02
//!
//! Assumes `miner` and `receiver` wallets already exist (created earlier via
//! `bitcoin-cli createwallet` in Polar's terminal), so this doesn't call
//! `create_wallet` again (that would error on an already-existing wallet).

use rfb_labs_week_1::labs::lab02_wallets::{
    address_belongs_to_wallet, get_new_address, list_wallets,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let client = ProcessRpc::new("docker").with_base_args([
        "exec",
        "-u",
        "bitcoin",
        "polar-n1-backend1",
        "bitcoin-cli",
        "-regtest",
    ]);

    let wallets = list_wallets(&client).expect("list_wallets failed");
    println!("Loaded wallets: {wallets:?}");

    let miner_address =
        get_new_address(&client, "miner", "mining-rust").expect("get_new_address (miner) failed");
    println!("miner address: {miner_address}");

    let receiver_address = get_new_address(&client, "receiver", "classmate-rust")
        .expect("get_new_address (receiver) failed");
    println!("receiver address: {receiver_address}");

    let miner_owns_own = address_belongs_to_wallet(&client, "miner", &miner_address)
        .expect("ownership check failed");
    let receiver_owns_miner = address_belongs_to_wallet(&client, "receiver", &miner_address)
        .expect("ownership check failed");

    println!("miner wallet owns its own address: {miner_owns_own}");
    println!("receiver wallet owns miner's address: {receiver_owns_miner}");
}
