//! Lab 05 evidence runner.
//!
//! Usage: cargo run --example lab05 [-- <receiver_address>]
//!
//! Sends exactly 1 BTC from miner to receiver without mining, then proves the
//! payment is broadcast but unconfirmed: it's in the mempool, the sender sees
//! zero confirmations, and the receiver sees an untrusted-pending balance.

use rfb_labs_week_1::labs::lab05_mempool::observe_unconfirmed_payment;
use rfb_labs_week_1::rpc::ProcessRpc;

const DEFAULT_RECEIVER_ADDRESS: &str = "bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn";

fn main() {
    let receiver_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_RECEIVER_ADDRESS.to_owned());

    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let observation =
        observe_unconfirmed_payment(&rpc, "miner", "receiver", &receiver_address, 1.0)
            .expect("observe unconfirmed payment");

    println!("txid:                 {}", observation.txid);
    println!("mempool contains tx:  {}", observation.mempool_contains_tx);
    println!(
        "sender confirmations: {}",
        observation.sender_status.confirmations
    );
    println!("sender amount:        {}", observation.sender_status.amount);
    println!("sender fee:           {:?}", observation.sender_status.fee);
    println!(
        "sender block_hash:    {:?}",
        observation.sender_status.block_hash
    );
    println!(
        "receiver balance: trusted={} untrusted_pending={} immature={}",
        observation.receiver_balance.trusted,
        observation.receiver_balance.untrusted_pending,
        observation.receiver_balance.immature
    );
}
