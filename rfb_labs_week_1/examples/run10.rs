use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab10_reorg::{
    build_reorg_report, disconnect_peer, get_chain_tip, reconnect_peer,
};
use rfb_labs_week_1::model::ForkSnapshot;
use rfb_labs_week_1::rpc::{ProcessRpc, RpcClient};
use std::thread::sleep;
use std::time::Duration;

fn first_peer_addr<C: RpcClient>(client: &C) -> Option<String> {
    let raw = client.call(None, "getpeerinfo", &[]).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value
        .as_array()?
        .first()?
        .get("addr")?
        .as_str()
        .map(ToOwned::to_owned)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node_a = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcconnect=127.0.0.1",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);
    let node_b = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcconnect=127.0.0.1",
        "-rpcport=18444",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let address = "bcrt1qgma38ugkn7lgyr0tf2k7pp5shfrn7gq8v5kgsg";

    println!("=== Lab 10: common tip before split ===");
    let common_tip = get_chain_tip(&node_a)?;
    println!("{common_tip:#?}");
    let common_tip_before_split = common_tip.best_block_hash.clone();

    println!("\n=== Disconnecting peers ===");
    if let Some(addr) = first_peer_addr(&node_a) {
        println!("node A disconnecting: {addr}");
        let _ = disconnect_peer(&node_a, &addr);
    } else {
        println!("node A: already had no peers");
    }
    if let Some(addr) = first_peer_addr(&node_b) {
        println!("node B disconnecting: {addr}");
        let _ = disconnect_peer(&node_b, &addr);
    } else {
        println!("node B: already had no peers");
    }

    println!("\n=== Mining privately ===");
    mine_blocks(&node_a, address, 2)?;
    mine_blocks(&node_b, address, 4)?;

    let tip_a = get_chain_tip(&node_a)?;
    let tip_b = get_chain_tip(&node_b)?;
    println!("node A private tip: {tip_a:#?}");
    println!("node B private tip: {tip_b:#?}");

    println!("\n=== Reconnecting ===");
    reconnect_peer(&node_a, "backend2")?;
    reconnect_peer(&node_b, "backend1")?;
    sleep(Duration::from_secs(6));

    let final_a = get_chain_tip(&node_a)?;
    let final_b = get_chain_tip(&node_b)?;
    println!("node A final tip: {final_a:#?}");
    println!("node B final tip: {final_b:#?}");

    let report = build_reorg_report(
        &common_tip_before_split,
        ForkSnapshot {
            node_a: tip_a,
            node_b: tip_b,
        },
        ForkSnapshot {
            node_a: final_a,
            node_b: final_b,
        },
    );
    println!("\n=== Reorg report ===");
    println!("{report:#?}");

    Ok(())
}
