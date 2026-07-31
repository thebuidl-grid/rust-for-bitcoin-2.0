use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab10_reorg::{
    build_reorg_report, disconnect_peer, get_chain_tip, reconnect_peer,
};
use rfb_labs_week_1::model::ForkSnapshot;
use rfb_labs_week_1::rpc::ProcessRpc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lab 10: Blockchain Reorganization ===\n");

    // Node A (default Polar node on port 18443)
    let node_a = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18445",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    // Node B (second Polar node on port 18444)
    let node_b = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18444",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Step 1: Setting up wallets on both nodes...\n");

    for (node, name) in [(&node_a, "Node A"), (&node_b, "Node B")] {
        match create_wallet(node, "miner") {
            Ok(_) => println!("  ✓ Created miner wallet on {}", name),
            Err(_) => println!("  ℹ Using existing miner wallet on {}", name),
        }
    }
    println!();

    println!("Step 2: Mining initial blocks for synchronization...\n");
    let miner_address_a = get_new_address(&node_a, "miner", "mining")?;
    mine_blocks(&node_a, &miner_address_a, 101)?;
    println!("  ✓ Node A mined 101 blocks\n");

    // Wait for Node B to sync
    println!("Step 3: Waiting for Node B to synchronize...\n");
    thread::sleep(Duration::from_secs(3));
    let tip_b = get_chain_tip(&node_b)?;
    println!("  ✓ Node B synced to height {}\n", tip_b.height);

    // Record common tip
    println!("Step 4: Recording common chain tip before split...\n");
    let common_tip_a = get_chain_tip(&node_a)?;
    let common_tip_b = get_chain_tip(&node_b)?;
    println!(
        "  Node A: height {}, hash {}",
        common_tip_a.height,
        &common_tip_a.best_block_hash[..16]
    );
    println!(
        "  Node B: height {}, hash {}",
        common_tip_b.height,
        &common_tip_b.best_block_hash[..16]
    );

    if common_tip_a.best_block_hash != common_tip_b.best_block_hash {
        println!("\n  ⚠ Warning: Nodes not fully synced yet. Waiting...");
        thread::sleep(Duration::from_secs(5));
    }

    let common_hash = common_tip_a.best_block_hash.clone();
    println!("  ✓ Common tip: {}\n", &common_hash[..16]);

    // Disconnect the nodes
    println!("Step 5: Disconnecting nodes to create competing chains...\n");

    // Get Node B's address from Node A's perspective
    // In Polar, Node B is typically at 127.0.0.1:19444 (P2P port, not RPC port)
    let node_b_address = "backend2";

    disconnect_peer(&node_a, node_b_address)?;
    println!("  ✓ Nodes disconnected\n");

    // Mine 2 blocks on Node A
    println!("Step 6: Mining 2 blocks privately on Node A...\n");
    mine_blocks(&node_a, &miner_address_a, 2)?;
    let private_tip_a = get_chain_tip(&node_a)?;
    println!(
        "  Node A: height {}, hash {}",
        private_tip_a.height,
        &private_tip_a.best_block_hash[..16]
    );
    println!("  Chainwork: {}\n", private_tip_a.chainwork);

    // Mine 4 blocks on Node B
    println!("Step 7: Mining 4 blocks privately on Node B...\n");
    let miner_address_b = get_new_address(&node_b, "miner", "mining")?;
    mine_blocks(&node_b, &miner_address_b, 4)?;
    let private_tip_b = get_chain_tip(&node_b)?;
    println!(
        "  Node B: height {}, hash {}",
        private_tip_b.height,
        &private_tip_b.best_block_hash[..16]
    );
    println!("  Chainwork: {}\n", private_tip_b.chainwork);

    // Record competing tips
    let competing_tips = ForkSnapshot {
        node_a: private_tip_a.clone(),
        node_b: private_tip_b.clone(),
    };

    println!("=== Competing Chains ===\n");
    println!(
        "Node A (shorter): height {}, work {}",
        competing_tips.node_a.height, competing_tips.node_a.chainwork
    );
    println!(
        "Node B (longer):  height {}, work {}\n",
        competing_tips.node_b.height, competing_tips.node_b.chainwork
    );

    // Reconnect the nodes
    println!("Step 8: Reconnecting nodes for synchronization...\n");
    reconnect_peer(&node_a, node_b_address)?;
    println!("  ✓ Reconnection initiated\n");

    // Wait for synchronization
    println!("Step 9: Waiting for reorganization to complete...\n");
    thread::sleep(Duration::from_secs(5));

    // Check final tips
    let final_tip_a = get_chain_tip(&node_a)?;
    let final_tip_b = get_chain_tip(&node_b)?;

    let final_tips = ForkSnapshot {
        node_a: final_tip_a.clone(),
        node_b: final_tip_b.clone(),
    };

    println!("=== Final Chain State ===\n");
    println!(
        "Node A: height {}, hash {}",
        final_tips.node_a.height,
        &final_tips.node_a.best_block_hash[..16]
    );
    println!("        chainwork {}", final_tips.node_a.chainwork);
    println!(
        "Node B: height {}, hash {}",
        final_tips.node_b.height,
        &final_tips.node_b.best_block_hash[..16]
    );
    println!("        chainwork {}\n", final_tips.node_b.chainwork);

    // Build reorg report
    let report = build_reorg_report(&common_hash, competing_tips, final_tips);

    println!("=== Reorganization Report ===\n");
    println!(
        "Common tip before split: {}",
        &report.common_tip_before_split[..16]
    );
    println!("\nCompeting branches:");
    println!(
        "  Node A: {} blocks, work {}",
        report.competing_tips.node_a.height, report.competing_tips.node_a.chainwork
    );
    println!(
        "  Node B: {} blocks, work {}",
        report.competing_tips.node_b.height, report.competing_tips.node_b.chainwork
    );
    println!("\nFinal state:");
    println!(
        "  Node A: {} blocks, hash {}",
        report.final_tips.node_a.height,
        &report.final_tips.node_a.best_block_hash[..16]
    );
    println!(
        "  Node B: {} blocks, hash {}",
        report.final_tips.node_b.height,
        &report.final_tips.node_b.best_block_hash[..16]
    );
    println!("\nConverged: {}\n", report.converged);

    if report.converged {
        println!("=== Verification ===\n");
        println!("✓ Both nodes converged on the same chain");
        println!("✓ Node A reorganized to Node B's longer chain");
        println!("✓ Node B's 4 blocks had more work than Node A's 2 blocks");
        println!("✓ Node A's 2 private blocks became stale (orphaned)");
        println!("\n=== Key Insight ===");
        println!("Nodes choose the chain with the most accumulated work,");
        println!("not the first-seen, longest-by-count, or socially-preferred chain.");
        println!("This is Bitcoin's Nakamoto consensus.\n");
    } else {
        println!("⚠ Warning: Nodes did not converge. May need more sync time.");
    }

    println!("=== Lab 10 Complete ===");

    Ok(())
}
