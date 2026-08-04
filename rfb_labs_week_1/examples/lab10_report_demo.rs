//! Builds the final Lab 10 report from the already-recorded tip data
//! (see submissions/lab_10.md for how disconnect/mine/reconnect were run
//! manually after the first attempt revealed a persistent `addnode` entry
//! auto-reconnecting the two nodes mid-mine), and independently re-reads
//! both nodes' final tips live via `get_chain_tip` to confirm convergence.
//!
//! Usage: BITCOIN_CLI_A=/path/to/node-a-wrapper BITCOIN_CLI_B=/path/to/node-b-wrapper \
//!        cargo run --example lab10_report_demo

use rfb_labs_week_1::labs::lab10_reorg::{build_reorg_report, get_chain_tip};
use rfb_labs_week_1::model::ChainTip;
use rfb_labs_week_1::rpc::ProcessRpc;

fn tip(height: u64, hash: &str, chainwork: &str) -> ChainTip {
    ChainTip {
        height,
        best_block_hash: hash.to_string(),
        chainwork: chainwork.to_string(),
    }
}

fn main() {
    let binary_a = std::env::var("BITCOIN_CLI_A").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let binary_b = std::env::var("BITCOIN_CLI_B").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let node_a = ProcessRpc::new(binary_a);
    let node_b = ProcessRpc::new(binary_b);

    let competing_tips = rfb_labs_week_1::model::ForkSnapshot {
        node_a: tip(
            120,
            "101674d0264507d0ad5038fa4310e25e88f80a92215cbfa03b63eed24082e5bf",
            "00000000000000000000000000000000000000000000000000000000000000f2",
        ),
        node_b: tip(
            122,
            "173bddc09617c0412d24c58ccaa8be5c46047f38d99bc04633ea17c1cbeac643",
            "00000000000000000000000000000000000000000000000000000000000000f6",
        ),
    };

    let final_tip_a = get_chain_tip(&node_a).expect("node A tip failed");
    let final_tip_b = get_chain_tip(&node_b).expect("node B tip failed");
    println!("live final tip (node A) = {final_tip_a:?}");
    println!("live final tip (node B) = {final_tip_b:?}");

    let final_tips = rfb_labs_week_1::model::ForkSnapshot {
        node_a: final_tip_a,
        node_b: final_tip_b,
    };

    let report = build_reorg_report(
        "2311a984f93770e7fce928fb8b5fb11d65405e539f9defd4950a9c674e1585ed",
        competing_tips,
        final_tips,
    );
    println!("\n{report:#?}");
}
