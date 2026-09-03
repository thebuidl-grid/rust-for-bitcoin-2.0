use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::{get_balances, mine_blocks};
use rfb_labs_week_1::labs::lab05_mempool::{get_raw_mempool, send_btc};
use rfb_labs_week_1::labs::lab07_confirm::confirm_and_locate_transaction;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    println!("=== Lab 07: Transaction Confirmation ===\n");

    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18445",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Step 1: Setting up wallets...\n");

    for wallet in &["miner", "receiver"] {
        match create_wallet(&rpc, wallet) {
            Ok(_) => println!("  ✓ Created {} wallet", wallet),
            Err(e) if e.to_string().contains("already exists") => {
                println!("  ℹ Using existing {} wallet", wallet)
            }
            Err(e) => {
                eprintln!("  ✗ Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("\nStep 2: Ensuring miner has funds...\n");

    let miner_address = get_new_address(&rpc, "miner", "mining").unwrap();
    mine_blocks(&rpc, &miner_address, 101).unwrap();

    let balance = get_balances(&rpc, "miner").unwrap();
    println!("  ✓ Miner has {} BTC", balance.trusted);

    println!("\nStep 3: Sending unconfirmed payment...\n");

    let receiver_address = get_new_address(&rpc, "receiver", "payment").unwrap();
    println!("  Receiver address: {}", receiver_address);

    let txid = send_btc(&rpc, "miner", &receiver_address, 2.0).unwrap();
    println!("  ✓ Payment sent: {}", txid);

    let mempool_before = get_raw_mempool(&rpc).unwrap();
    println!("  ℹ Mempool contains {} transactions", mempool_before.len());

    println!("\nStep 4: Confirming transaction by mining...\n");

    match confirm_and_locate_transaction(&rpc, "miner", &txid, &miner_address) {
        Ok(report) => {
            println!("=== Confirmation Report ===\n");
            println!("Transaction ID:     {}", report.txid);
            println!("Block Hash:         {}", report.block_hash);
            println!("Confirmations:      {}", report.confirmations);
            println!("Mempool Empty:      {}", report.mempool_is_empty);
            println!("In Block:           {}", report.transaction_is_in_block);

            println!("\n=== Verification ===");

            if report.confirmations >= 1 {
                println!("✓ Transaction has {} confirmation(s)", report.confirmations);
            } else {
                println!("✗ Transaction not confirmed!");
            }

            if report.mempool_is_empty {
                println!("✓ Mempool is empty (all transactions confirmed)");
            } else {
                println!("ℹ Mempool still has unconfirmed transactions");
            }

            if report.transaction_is_in_block {
                println!("✓ Transaction found in block {}", report.block_hash);
            } else {
                println!("✗ Transaction NOT found in block!");
            }

            println!("\n=== What Happened ===");
            println!("1. Transaction was broadcast → entered mempool");
            println!("2. Block was mined → transaction left mempool");
            println!("3. Transaction now has 1 confirmation");
            println!(
                "4. Transaction is permanently in block {}",
                &report.block_hash[..16]
            );
            println!("5. Each new block adds another confirmation");

            println!("\n=== Lab 07 Complete ===");
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
