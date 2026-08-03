use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab10_reorg::{
    build_reorg_report, disconnect_peer, get_chain_tip, reconnect_peer,
};
use rfb_labs_week_1::model::ForkSnapshot;
use rfb_labs_week_1::rpc::ProcessRpc;
use std::thread;
use std::time::Duration;

fn main() {
    let node_a = ProcessRpc::new("docker").with_base_args([
        "exec",
        "polar-n3-backend1",
        "bitcoin-cli",
        "-regtest",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);
    let node_b = ProcessRpc::new("docker").with_base_args([
        "exec",
        "polar-n3-backend2",
        "bitcoin-cli",
        "-regtest",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let node_a_mining_address = "bcrt1qdkkp53s2ltt48adua2yv0qcxqsc9m6h0at63vr";
    let node_b_mining_address = "bcrt1q36rp0mvfy4sc5lly75hvqyhj3p2gzhv6nd6lgm";
    let node_b_peer_addr_from_a = "172.20.0.4:58866";

    let common_tip = get_chain_tip(&node_a).unwrap();
    println!("common tip before split: {common_tip:#?}");
    assert_eq!(common_tip, get_chain_tip(&node_b).unwrap());

    disconnect_peer(&node_a, node_b_peer_addr_from_a).unwrap();
    println!("disconnected node A from node B");

    mine_blocks(&node_a, node_a_mining_address, 2).unwrap();
    mine_blocks(&node_b, node_b_mining_address, 4).unwrap();

    let competing_tips = ForkSnapshot {
        node_a: get_chain_tip(&node_a).unwrap(),
        node_b: get_chain_tip(&node_b).unwrap(),
    };
    println!("competing tips: {competing_tips:#?}");

    reconnect_peer(&node_b, "backend1").unwrap();
    println!("reconnected node B to node A, waiting for sync...");

    let mut final_tips;
    loop {
        thread::sleep(Duration::from_millis(500));
        final_tips = ForkSnapshot {
            node_a: get_chain_tip(&node_a).unwrap(),
            node_b: get_chain_tip(&node_b).unwrap(),
        };
        if final_tips.node_a == final_tips.node_b {
            break;
        }
    }

    let report = build_reorg_report(&common_tip.best_block_hash, competing_tips, final_tips);
    println!("{report:#?}");
}
