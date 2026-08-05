//! Lab 10 evidence runner.
//!
//! Usage:
//!   cargo run --example lab10 -- <node_b_rpc_port> <disconnect_address> <reconnect_address> <node_a_miner_address> <node_b_miner_address>
//!
//! Requires a second Bitcoin Core node already added to your Polar network.
//! Polar auto-connects nodes within the same network. Run, against node A:
//!   bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass getpeerinfo
//!
//! For an inbound connection (node B dialed node A), the "addr" field is an
//! ephemeral source port, not node B's listener — so two different addresses
//! are needed:
//!   <disconnect_address> = the exact "addr" field from getpeerinfo
//!                           (matches the live connection to drop it)
//!   <reconnect_address>  = "<same IP>:18444" (regtest's default P2P port,
//!                           node B's actual listening address)

use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab10_reorg::{
    build_reorg_report, disconnect_peer, get_chain_tip, reconnect_peer,
};
use rfb_labs_week_1::model::ForkSnapshot;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let mut args = std::env::args().skip(1);
    let node_b_rpc_port = args.next().expect(
        "usage: cargo run --example lab10 -- <node_b_rpc_port> <disconnect_address> <reconnect_address> <node_a_miner_address> <node_b_miner_address>",
    );
    let disconnect_address = args
        .next()
        .expect("disconnect_address required (see getpeerinfo)");
    let reconnect_address = args
        .next()
        .expect("reconnect_address required (<node_b_ip>:18444)");
    let node_a_miner_address = args.next().expect("node_a_miner_address required");
    let node_b_miner_address = args.next().expect("node_b_miner_address required");

    let node_a = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);
    let node_b = ProcessRpc::new("bitcoin-cli").with_base_args(vec![
        "-regtest".to_owned(),
        format!("-rpcport={node_b_rpc_port}"),
        "-rpcuser=polaruser".to_owned(),
        "-rpcpassword=polarpass".to_owned(),
    ]);

    let common_tip_a = get_chain_tip(&node_a).expect("node A tip before split");
    let common_tip_b = get_chain_tip(&node_b).expect("node B tip before split");
    assert_eq!(
        common_tip_a, common_tip_b,
        "nodes were not synchronized before the split"
    );
    println!(
        "common tip before split: height={} hash={} chainwork={}",
        common_tip_a.height, common_tip_a.best_block_hash, common_tip_a.chainwork
    );

    disconnect_peer(&node_a, &disconnect_address).expect("disconnect peer from node A");

    mine_blocks(&node_a, &node_a_miner_address, 2).expect("mine 2 private blocks on node A");
    mine_blocks(&node_b, &node_b_miner_address, 4).expect("mine 4 private blocks on node B");

    let competing_tip_a = get_chain_tip(&node_a).expect("node A private tip");
    let competing_tip_b = get_chain_tip(&node_b).expect("node B private tip");
    println!(
        "node A private tip: height={} hash={} chainwork={}",
        competing_tip_a.height, competing_tip_a.best_block_hash, competing_tip_a.chainwork
    );
    println!(
        "node B private tip: height={} hash={} chainwork={}",
        competing_tip_b.height, competing_tip_b.best_block_hash, competing_tip_b.chainwork
    );

    reconnect_peer(&node_a, &reconnect_address).expect("reconnect peer to node A");

    let mut final_tip_a = get_chain_tip(&node_a).expect("node A final tip");
    let mut final_tip_b = get_chain_tip(&node_b).expect("node B final tip");
    for _ in 0..10 {
        if final_tip_a.best_block_hash == final_tip_b.best_block_hash {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        final_tip_a = get_chain_tip(&node_a).expect("node A final tip");
        final_tip_b = get_chain_tip(&node_b).expect("node B final tip");
    }

    let report = build_reorg_report(
        &common_tip_a.best_block_hash,
        ForkSnapshot {
            node_a: competing_tip_a,
            node_b: competing_tip_b,
        },
        ForkSnapshot {
            node_a: final_tip_a,
            node_b: final_tip_b,
        },
    );

    println!("converged: {}", report.converged);
    println!(
        "final node A tip: height={} hash={}",
        report.final_tips.node_a.height, report.final_tips.node_a.best_block_hash
    );
    println!(
        "final node B tip: height={} hash={}",
        report.final_tips.node_b.height, report.final_tips.node_b.best_block_hash
    );
}
