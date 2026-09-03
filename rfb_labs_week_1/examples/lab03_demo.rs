use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::demonstrate_coinbase_maturity;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    println!("=== Lab 03: Coinbase Maturity Demonstration ===\n");

    // Create RPC client pointing to Polar's regtest node
    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18445",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Step 1: Setting up miner wallet...");

    // Create miner wallet (or use existing)
    match create_wallet(&rpc, "miner") {
        Ok(_) => println!("  ✓ Created miner wallet"),
        Err(e) => {
            if e.to_string().contains("already exists") {
                println!("  ℹ Using existing miner wallet");
            } else {
                eprintln!("  ✗ Error creating miner wallet: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("\nStep 2: Generating addresses...");

    // Generate address for mining rewards
    let miner_address = match get_new_address(&rpc, "miner", "coinbase-rewards") {
        Ok(addr) => {
            println!("  ✓ Miner address: {}", addr);
            addr
        }
        Err(e) => {
            eprintln!("  ✗ Error generating miner address: {}", e);
            std::process::exit(1);
        }
    };

    // Use a dummy receiver address (we don't need a real wallet for the failed payment test)
    let receiver_address = "bcrt1qreceiver00000000000000000000000000test".to_string();
    println!("  ✓ Using test receiver address: {}", receiver_address);

    println!("\n=== Starting Coinbase Maturity Test ===\n");
    println!("This will:");
    println!("  1. Mine 1 block to miner address");
    println!("  2. Check balances (reward should be IMMATURE)");
    println!("  3. Try to spend 1 BTC (should FAIL - insufficient funds)");
    println!("  4. Mine 100 more blocks (total 101)");
    println!("  5. Check balances (first reward should now be TRUSTED/spendable)");
    println!("\nThis may take a moment...\n");

    // Run the demonstration
    match demonstrate_coinbase_maturity(&rpc, "miner", &miner_address, &receiver_address) {
        Ok(report) => {
            println!("=== Coinbase Maturity Report ===\n");

            println!("After mining block 1:");
            println!("  Block Height:        {}", report.height_after_first_block);
            println!(
                "  Trusted Balance:     {} BTC",
                report.balance_after_first_block.trusted
            );
            println!(
                "  Untrusted Pending:   {} BTC",
                report.balance_after_first_block.untrusted_pending
            );
            println!(
                "  Immature Balance:    {} BTC",
                report.balance_after_first_block.immature
            );

            println!("\nAttempted to spend 1 BTC:");
            println!("  ✗ Error (expected): {}", report.premature_spend_error);

            println!("\nAfter mining 100 more blocks (total 101):");
            println!("  Block Height:        {}", report.final_height);
            println!(
                "  Trusted Balance:     {} BTC",
                report.final_balance.trusted
            );
            println!(
                "  Untrusted Pending:   {} BTC",
                report.final_balance.untrusted_pending
            );
            println!(
                "  Immature Balance:    {} BTC",
                report.final_balance.immature
            );

            println!("\n=== Key Observations ===");
            println!(
                "✓ Block 1 reward was IMMATURE ({} BTC)",
                report.balance_after_first_block.immature
            );
            println!("✓ Spending failed with: '{}'", report.premature_spend_error);
            println!(
                "✓ At block 101, first reward became TRUSTED ({} BTC)",
                report.final_balance.trusted
            );
            println!(
                "✓ Newer rewards still IMMATURE ({} BTC from blocks 2-101)",
                report.final_balance.immature
            );

            println!("\n=== Coinbase Maturity Proven! ===");
            println!("Coinbase rewards require 100 confirmations before they can be spent.");
            println!("The reward from block 1 became spendable at block 101.");
        }
        Err(e) => {
            eprintln!("\n✗ Error during demonstration: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("  1. Make sure Polar network is running");
            eprintln!("  2. Try unloading extra wallets:");
            eprintln!("     bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass unloadwallet \"receiver\"");
            eprintln!("     bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass unloadwallet \"\"");
            eprintln!("  3. Ensure only 'miner' wallet is loaded");
            std::process::exit(1);
        }
    }
}
