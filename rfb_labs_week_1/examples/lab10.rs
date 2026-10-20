//! Runs the Lab 10 functions against two real Polar/regtest nodes.
//!
//! `disconnect_peer`/`reconnect_peer` are demonstrated directly (matching
//! what the graded functions do), but Polar's nodes are configured to
//! auto-reconnect almost immediately, so `setnetworkactive false`/`true`
//! (plain RPC calls, not part of the graded functions) are used around the
//! private-mining phase to guarantee real isolation while each side mines.
//!
//! Run with: cargo run --example lab10

use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab10_reorg::{
    build_reorg_report, disconnect_peer, get_chain_tip, reconnect_peer,
};
use rfb_labs_week_1::model::ForkSnapshot;
use rfb_labs_week_1::rpc::{ProcessRpc, RpcClient};
use serde_json::Value;

fn node_client(container: &str) -> ProcessRpc {
    ProcessRpc::new("docker").with_base_args([
        "exec",
        "-u",
        "bitcoin",
        container,
        "bitcoin-cli",
        "-regtest",
    ])
}

fn first_peer_address<C: RpcClient>(client: &C) -> Option<String> {
    let raw = client.call(None, "getpeerinfo", &[]).ok()?;
    let peers: Value = serde_json::from_str(&raw).ok()?;
    peers.get(0)?.get("addr")?.as_str().map(ToOwned::to_owned)
}

fn set_network_active<C: RpcClient>(client: &C, active: bool) {
    client
        .call(None, "setnetworkactive", &[active.to_string()])
        .expect("setnetworkactive failed");
}

fn main() {
    let node_a = node_client("polar-n1-backend1");
    let node_b = node_client("polar-n1-backend2");

    let common_tip = get_chain_tip(&node_a).expect("get_chain_tip (A, before split) failed");
    println!("common tip before split: {common_tip:#?}");

    if let Some(peer) = first_peer_address(&node_a) {
        disconnect_peer(&node_a, &peer).expect("disconnect_peer failed");
        println!("disconnect_peer called on A -> {peer}");
    }

    set_network_active(&node_a, false);
    set_network_active(&node_b, false);

    // Mine privately: 2 blocks on A, 4 on B.
    let a_mining_address = node_a
        .call(Some("miner"), "getnewaddress", &["reorg-demo-a".to_owned()])
        .expect("getnewaddress (A) failed");
    mine_blocks(&node_a, &a_mining_address, 2).expect("mine_blocks (A) failed");

    let b_mining_address = node_b
        .call(Some("nodeb"), "getnewaddress", &["reorg-demo-b".to_owned()])
        .expect("getnewaddress (B) failed");
    mine_blocks(&node_b, &b_mining_address, 4).expect("mine_blocks (B) failed");

    let competing_tips = ForkSnapshot {
        node_a: get_chain_tip(&node_a).expect("get_chain_tip (A, competing) failed"),
        node_b: get_chain_tip(&node_b).expect("get_chain_tip (B, competing) failed"),
    };
    println!("competing tips: {competing_tips:#?}");

    set_network_active(&node_a, true);
    set_network_active(&node_b, true);
    reconnect_peer(&node_a, "backend2:18444").expect("reconnect_peer failed");
    println!("reconnect_peer called on A -> backend2:18444");

    std::thread::sleep(std::time::Duration::from_secs(3));

    let final_tips = ForkSnapshot {
        node_a: get_chain_tip(&node_a).expect("get_chain_tip (A, final) failed"),
        node_b: get_chain_tip(&node_b).expect("get_chain_tip (B, final) failed"),
    };
    println!("final tips: {final_tips:#?}");

    let report = build_reorg_report(&common_tip.best_block_hash, competing_tips, final_tips);
    println!("{report:#?}");
}
