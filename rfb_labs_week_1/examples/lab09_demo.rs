use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use rfb_labs_week_1::labs::lab07_confirm::mine_one_block;
use rfb_labs_week_1::labs::lab09_coin_selection::{
    confirmed_utxos_for_address, create_three_funding_transactions, send_combined_payment,
};
use rfb_labs_week_1::model::OutPoint;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lab 09: Multi-UTXO Coin Selection ===\n");

    let client = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18445",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Step 1: Setting up wallets...\n");

    match create_wallet(&client, "miner") {
        Ok(_) => println!("  ✓ Created miner wallet"),
        Err(_) => println!("  ℹ Using existing miner wallet"),
    }

    match create_wallet(&client, "alice") {
        Ok(_) => println!("  ✓ Created alice wallet"),
        Err(_) => println!("  ℹ Using existing alice wallet"),
    }

    let miner_address = get_new_address(&client, "miner", "mining")?;
    println!("  ✓ Miner address: {}\n", miner_address);

    println!("Step 2: Mining 101 blocks for spendable balance...\n");
    mine_blocks(&client, &miner_address, 101)?;
    println!("  ✓ Mined 101 blocks\n");

    println!("Step 3: Creating Alice's address...\n");
    let alice_address = get_new_address(&client, "alice", "funding")?;
    println!("  ✓ Alice address: {}\n", alice_address);

    println!("Step 4: Sending three separate 0.4 BTC payments to Alice...\n");
    let funding_txids = create_three_funding_transactions(&client, "miner", &alice_address)?;
    for (i, txid) in funding_txids.iter().enumerate() {
        println!("  ✓ Funding Transaction {}: {}", i + 1, &txid[..16]);
    }
    println!();

    println!("Step 5: Mining one block to confirm funding...\n");
    mine_one_block(&client, &miner_address)?;
    println!("  ✓ Funding transactions confirmed\n");

    println!("Step 6: Listing Alice's confirmed UTXOs...\n");
    let alice_utxos = confirmed_utxos_for_address(&client, "alice", &alice_address)?;
    println!("  Alice has {} confirmed UTXO(s):", alice_utxos.len());
    for (i, utxo) in alice_utxos.iter().enumerate() {
        println!(
            "    UTXO {}: {} BTC ({}:{})",
            i + 1,
            utxo.amount,
            &utxo.txid[..16],
            utxo.vout
        );
    }
    let total: f64 = alice_utxos.iter().map(|u| u.amount).sum();
    println!("  Total: {} BTC\n", total);

    println!("Step 7: Creating receiver address...\n");
    let receiver_address = get_new_address(&client, "miner", "receiver")?;
    println!("  ✓ Receiver address: {}\n", receiver_address);

    println!("Step 8: Alice sending 1 BTC (requires multiple UTXOs)...\n");
    let spend_txid = send_combined_payment(&client, "alice", &receiver_address)?;
    println!("  ✓ Spend Transaction: {}\n", spend_txid);

    println!("Step 9: Mining block to enable transaction decoding...\n");
    mine_one_block(&client, &miner_address)?;
    println!("  ✓ Block mined\n");

    println!("Step 10: Decoding and auditing the multi-UTXO spend...\n");
    let decoded_tx = decode_verbose_transaction(&client, &spend_txid)?;
    let payment_and_change = identify_payment_and_change(&decoded_tx, &receiver_address)?;
    let raw_fee = calculate_fee(&decoded_tx)?;
    let fee = (raw_fee * 100_000_000.0).round() / 100_000_000.0;
    let funding_outpoints: Vec<OutPoint> = alice_utxos.iter().map(|utxo| utxo.outpoint()).collect();

    println!("=== Multi-UTXO Spend Audit ===\n");
    println!("Spend Transaction: {}", spend_txid);
    println!("Input Count:       {}", decoded_tx.inputs.len());
    println!("\nFunding Outpoints (inputs consumed):");
    for (i, outpoint) in funding_outpoints.iter().enumerate() {
        println!(
            "  Input {}: {}:{}",
            i + 1,
            &outpoint.txid[..16],
            outpoint.vout
        );
    }

    println!("\nOutputs:");
    println!(
        "  Payment:  {} BTC → {}",
        payment_and_change.payment.value,
        payment_and_change
            .payment
            .address
            .as_ref()
            .unwrap_or(&"(none)".to_string())
    );

    if let Some(change) = &payment_and_change.change {
        println!(
            "  Change:   {} BTC → {}",
            change.value,
            change.address.as_ref().unwrap_or(&"(none)".to_string())
        );
    }

    println!("\nValue Conservation:");
    let output_total = payment_and_change.payment.value
        + payment_and_change
            .change
            .as_ref()
            .map(|c| c.value)
            .unwrap_or(0.0);
    println!("  Inputs:  {} BTC (3 UTXOs × 0.4 BTC)", total);
    println!("  Outputs: {} BTC", output_total);
    println!("  Fee:     {} BTC", fee);
    println!("  Balance: {} BTC\n", total - output_total - fee);

    println!("=== Verification ===\n");
    if decoded_tx.inputs.len() == 3 {
        println!("✓ Multiple inputs required (3 UTXOs combined)");
    }
    if payment_and_change.payment.value == 1.0 {
        println!("✓ Receiver got exactly 1.0 BTC");
    }
    if payment_and_change.change.is_some() {
        println!("✓ Change returned to Alice");
    }
    if fee > 0.0 && fee < 0.001 {
        println!("✓ Reasonable fee charged ({} BTC)", fee);
    }

    println!("\n=== Privacy Warning ===");
    println!("All 3 input UTXOs are now linked on the blockchain.");
    println!("This reveals they belong to the same owner (Alice).");
    println!("Common input ownership heuristic applies.\n");

    println!("=== Lab 09 Complete ===");

    Ok(())
}
