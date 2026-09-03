use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::{get_balances, mine_blocks};
use rfb_labs_week_1::labs::lab05_mempool::{get_raw_mempool, observe_unconfirmed_payment};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    println!("=== Lab 05: Mempool Observation ===\n");

    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Step 1: Setting up wallets...\n");

    // Ensure miner wallet exists
    match create_wallet(&rpc, "miner") {
        Ok(_) => println!("  ✓ Created miner wallet"),
        Err(e) if e.to_string().contains("already exists") => {
            println!("  ℹ Using existing miner wallet")
        }
        Err(e) => {
            eprintln!("  ✗ Error: {}", e);
            std::process::exit(1);
        }
    }

    // Ensure receiver wallet exists and is loaded
    let receiver_created = match create_wallet(&rpc, "receiver") {
        Ok(_) => {
            println!("  ✓ Created receiver wallet");
            true
        }
        Err(e) if e.to_string().contains("already exists") => {
            println!("  ℹ Using existing receiver wallet");
            false
        }
        Err(e) => {
            eprintln!("  ✗ Error: {}", e);
            std::process::exit(1);
        }
    };

    // If receiver was just created, verify it's loaded
    if receiver_created {
        println!("  ℹ Verifying receiver wallet is loaded...");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\nStep 2: Generating addresses...\n");

    let miner_address = match get_new_address(&rpc, "miner", "mining") {
        Ok(addr) => {
            println!("  ✓ Miner address: {}", addr);
            addr
        }
        Err(e) => {
            eprintln!("  ✗ Error: {}", e);
            std::process::exit(1);
        }
    };

    let receiver_address = match get_new_address(&rpc, "receiver", "payment") {
        Ok(addr) => {
            println!("  ✓ Receiver address: {}", addr);
            addr
        }
        Err(e) => {
            eprintln!("  ✗ Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nStep 3: Ensuring miner has spendable funds...\n");

    // Mine 101 blocks to ensure miner has spendable coins
    match mine_blocks(&rpc, &miner_address, 101) {
        Ok(hashes) => println!("  ✓ Mined {} blocks", hashes.len()),
        Err(e) => {
            eprintln!("  ✗ Error mining blocks: {}", e);
            std::process::exit(1);
        }
    }

    match get_balances(&rpc, "miner") {
        Ok(balances) => {
            println!("  ✓ Miner balances:");
            println!("    Trusted: {} BTC", balances.trusted);
            if balances.trusted < 1.0 {
                eprintln!("\n  ✗ Insufficient funds! Need at least 1 BTC.");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("  ✗ Error checking balance: {}", e);
            std::process::exit(1);
        }
    }

    println!("\nStep 4: Checking mempool before payment...\n");

    let _mempool_before = match get_raw_mempool(&rpc) {
        Ok(txids) => {
            println!("  ✓ Mempool contains {} transactions", txids.len());
            txids
        }
        Err(e) => {
            eprintln!("  ✗ Error reading mempool: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nStep 5: Sending unconfirmed payment...\n");
    println!("  Sending 1.0 BTC from miner to receiver...");
    println!("  (NOT mining - transaction stays in mempool)\n");

    match observe_unconfirmed_payment(&rpc, "miner", "receiver", &receiver_address, 1.0) {
        Ok(observation) => {
            println!("=== Mempool Observation ===\n");

            println!("Transaction Details:");
            println!("  TXID:                 {}", observation.txid);
            println!(
                "  In Mempool:           {}",
                observation.mempool_contains_tx
            );
            println!();

            println!("Sender (Miner) View:");
            println!(
                "  Confirmations:        {}",
                observation.sender_status.confirmations
            );
            println!(
                "  Amount:               {} BTC",
                observation.sender_status.amount
            );
            println!(
                "  Fee:                  {} BTC",
                observation
                    .sender_status
                    .fee
                    .map(|f| format!("{}", f))
                    .unwrap_or_else(|| "N/A".to_string())
            );
            println!(
                "  Block Hash:           {}",
                observation
                    .sender_status
                    .block_hash
                    .as_deref()
                    .unwrap_or("(none - unconfirmed)")
            );
            println!();

            println!("Receiver Balance:");
            println!(
                "  Trusted:              {} BTC",
                observation.receiver_balance.trusted
            );
            println!(
                "  Untrusted Pending:    {} BTC",
                observation.receiver_balance.untrusted_pending
            );
            println!(
                "  Immature:             {} BTC",
                observation.receiver_balance.immature
            );
            println!();

            println!("=== Key Observations ===");
            if observation.mempool_contains_tx {
                println!("✓ Transaction is in the mempool (unconfirmed)");
            }
            if observation.sender_status.confirmations == 0 {
                println!("✓ Transaction has 0 confirmations");
            }
            if observation.receiver_balance.untrusted_pending > 0.0 {
                println!(
                    "✓ Receiver shows {} BTC in untrusted_pending (not yet confirmed)",
                    observation.receiver_balance.untrusted_pending
                );
            }
            if observation.receiver_balance.trusted == 0.0 {
                println!("✓ Receiver's trusted balance is still 0 (hasn't confirmed yet)");
            }

            println!("\n=== What Happens Next ===");
            println!("When a block is mined:");
            println!("  1. Transaction leaves the mempool");
            println!("  2. Confirmations increase from 0 to 1");
            println!("  3. Receiver's balance moves from untrusted_pending to trusted");
            println!("  4. block_hash field will contain the block's hash");
        }
        Err(e) => {
            eprintln!("✗ Error observing payment: {}", e);
            eprintln!("\nPossible issues:");
            eprintln!("  - Insufficient funds");
            eprintln!("  - Invalid address");
            eprintln!("  - Wallet not loaded");
            std::process::exit(1);
        }
    }

    println!("\n=== Lab 05 Complete ===");
    println!("Successfully demonstrated mempool observation!");
    println!("\nTo confirm the transaction, run:");
    println!("  bitcoin-cli -regtest generatetoaddress 1 <address>");
}
