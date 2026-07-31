use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab05_mempool::send_btc;
use rfb_labs_week_1::labs::lab07_confirm::mine_one_block;
use rfb_labs_week_1::labs::lab08_security::{
    build_security_report, get_block_header, get_confirmations,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lab 08: Block Security and Proof-of-Work ===\n");

    let client = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18445",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Step 1: Setting up wallets and initial balance...\n");

    // Ensure miner wallet exists
    match create_wallet(&client, "miner") {
        Ok(_) => println!("  ✓ Created miner wallet"),
        Err(_) => println!("  ℹ Using existing miner wallet"),
    }

    // Create receiver wallet
    match create_wallet(&client, "receiver") {
        Ok(_) => println!("  ✓ Created receiver wallet"),
        Err(_) => println!("  ℹ Using existing receiver wallet"),
    }

    let miner_address = get_new_address(&client, "miner", "mining")?;
    println!("  ✓ Miner address: {}\n", miner_address);

    // Mine 101 blocks to get spendable balance
    println!("Step 2: Mining 101 blocks for spendable balance...\n");
    mine_blocks(&client, &miner_address, 101)?;
    println!("  ✓ Mined 101 blocks\n");

    // Send payment to receiver
    println!("Step 3: Sending payment to receiver...\n");
    let receiver_address = get_new_address(&client, "receiver", "payment")?;
    let txid = send_btc(&client, "miner", &receiver_address, 1.0)?;
    println!("  ✓ Payment TXID: {}\n", txid);

    // Mine one block to confirm the transaction
    println!("Step 4: Mining one block to confirm transaction...\n");
    let block_hash = mine_one_block(&client, &miner_address)?;
    println!("  ✓ Block Hash: {}\n", block_hash);

    // Inspect the block header
    println!("Step 5: Inspecting block header...\n");
    let header = get_block_header(&client, &block_hash)?;
    println!("  Block Header Evidence:");
    println!("    Hash:              {}", header.hash);
    println!("    Height:            {}", header.height);
    println!(
        "    Previous Hash:     {}",
        header
            .previous_block_hash
            .as_ref()
            .unwrap_or(&"(none)".to_string())
    );
    println!("    Merkle Root:       {}", header.merkle_root);
    println!("    Nonce:             {}", header.nonce);
    println!("    Difficulty:        {}", header.difficulty);
    println!("    Bits:              {}", header.bits);
    println!("    Confirmations:     {}", header.confirmations);
    println!("    Chainwork:         {}\n", header.chainwork);

    // Check initial confirmations
    println!("Step 6: Checking transaction confirmations...\n");
    let initial_confirmations = get_confirmations(&client, "receiver", &txid)?;
    println!("  Initial Confirmations: {}\n", initial_confirmations);

    // Build security report (mines 5 more blocks)
    println!("Step 7: Mining 5 additional blocks...\n");
    let report = build_security_report(&client, "receiver", &txid, &block_hash, &miner_address)?;

    println!("  ✓ Mined 5 additional blocks");
    println!("  Final Confirmations:   {}\n", report.confirmations_after);

    // Display security report
    println!("=== Security Report ===\n");
    println!("Transaction Security:");
    println!("  TXID:                  {}", txid);
    println!("  Block Hash:            {}", report.header.hash);
    println!("  Block Height:          {}", report.header.height);
    println!("  Merkle Root:           {}", report.header.merkle_root);
    println!("  Nonce:                 {}", report.header.nonce);
    println!("  Difficulty:            {}", report.header.difficulty);
    println!("  Bits (Target):         {}", report.header.bits);
    println!("  Chainwork:             {}", report.header.chainwork);
    println!("  Confirmations Before:  {}", report.confirmations_before);
    println!("  Confirmations After:   {}\n", report.confirmations_after);

    println!("=== Key Observations ===\n");
    println!("✓ Block header contains cryptographic proof of work");
    println!("✓ Merkle root commits to all transactions in the block");
    println!("✓ Nonce was found that produces valid block hash < target");
    println!(
        "✓ Confirmations increased from {} to {}",
        report.confirmations_before, report.confirmations_after
    );
    println!("✓ Each additional block increases reorganization cost\n");

    println!("=== Proof-of-Work Verified! ===\n");
    println!("Mining 5 blocks added 5 confirmations to the payment.");
    println!(
        "Reversing this transaction would require re-mining {} blocks.",
        report.confirmations_after
    );

    Ok(())
}
