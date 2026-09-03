use rfb_labs_week_1::labs::lab03_maturity::get_balances;
use rfb_labs_week_1::labs::lab04_utxos::{
    list_unspent, outpoint, select_spendable_utxo, sum_spendable_utxos,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    println!("=== Lab 04: UTXO Inspection ===\n");

    // Create RPC client pointing to Polar's regtest node
    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Step 1: Listing all UTXOs for miner wallet...\n");

    let utxos = match list_unspent(&rpc, "miner") {
        Ok(utxos) => {
            println!("  ✓ Found {} total UTXOs", utxos.len());
            utxos
        }
        Err(e) => {
            eprintln!("  ✗ Error listing UTXOs: {}", e);
            eprintln!("\nMake sure:");
            eprintln!("  1. Polar network is running");
            eprintln!("  2. Miner wallet exists and has been used");
            eprintln!("  3. Some blocks have been mined to the miner wallet");
            std::process::exit(1);
        }
    };

    if utxos.is_empty() {
        eprintln!("  ✗ No UTXOs found in miner wallet");
        eprintln!("\nYou need to mine some blocks first:");
        eprintln!("  Run Lab 03 demo or manually mine blocks to generate UTXOs");
        std::process::exit(1);
    }

    println!("\nAll UTXOs:");
    println!("{:-<100}", "");
    for (i, utxo) in utxos.iter().enumerate() {
        println!("UTXO #{}", i + 1);
        println!("  txid:           {}", utxo.txid);
        println!("  vout:           {}", utxo.vout);
        println!("  amount:         {} BTC", utxo.amount);
        println!("  confirmations:  {}", utxo.confirmations);
        println!(
            "  address:        {}",
            utxo.address.as_deref().unwrap_or("(none)")
        );
        println!("  scriptPubKey:   {}", utxo.script_pub_key);
        println!("  spendable:      {}", utxo.spendable);
        println!("{:-<100}", "");
    }

    println!("\nStep 2: Selecting one spendable UTXO...\n");

    let selected = match select_spendable_utxo(&utxos) {
        Some(utxo) => {
            println!("  ✓ Selected UTXO (most confirmed spendable):");
            println!("    txid:           {}", utxo.txid);
            println!("    vout:           {}", utxo.vout);
            println!("    amount:         {} BTC", utxo.amount);
            println!("    confirmations:  {}", utxo.confirmations);
            println!(
                "    address:        {}",
                utxo.address.as_deref().unwrap_or("(none)")
            );
            println!("    scriptPubKey:   {}", utxo.script_pub_key);
            println!("    spendable:      {}", utxo.spendable);
            utxo
        }
        None => {
            eprintln!("  ✗ No spendable UTXOs found!");
            eprintln!("\nAll UTXOs are immature (need 100 confirmations).");
            eprintln!("Mine more blocks to mature the coinbase rewards.");
            std::process::exit(1);
        }
    };

    println!("\nStep 3: Constructing outpoint...\n");

    let outpoint_id = outpoint(&selected);
    println!("  ✓ Outpoint (unique identifier):");
    println!("    {}:{}", outpoint_id.txid, outpoint_id.vout);
    println!("\n  This uniquely identifies this UTXO on the blockchain.");

    println!("\nStep 4: Summing all spendable UTXOs...\n");

    let spendable_sum = sum_spendable_utxos(&utxos);
    let spendable_count = utxos.iter().filter(|u| u.spendable).count();
    let immature_count = utxos.iter().filter(|u| !u.spendable).count();

    println!(
        "  ✓ Spendable UTXOs: {} out of {}",
        spendable_count,
        utxos.len()
    );
    println!("  ✓ Immature UTXOs:  {}", immature_count);
    println!("  ✓ Sum of spendable: {} BTC", spendable_sum);

    println!("\nStep 5: Reconciling with Bitcoin Core wallet balance...\n");

    match get_balances(&rpc, "miner") {
        Ok(balances) => {
            println!("  Bitcoin Core wallet balances:");
            println!("    Trusted:            {} BTC", balances.trusted);
            println!("    Untrusted Pending:  {} BTC", balances.untrusted_pending);
            println!("    Immature:           {} BTC", balances.immature);

            println!("\n  Reconciliation:");
            println!("    Our UTXO sum:       {} BTC", spendable_sum);
            println!("    Core trusted:       {} BTC", balances.trusted);

            let diff = (spendable_sum - balances.trusted).abs();
            if diff < 0.000_000_01 {
                println!("    ✓ MATCH! Balances reconcile correctly.");
            } else {
                println!("    ✗ MISMATCH! Difference: {} BTC", diff);
                println!(
                    "    (Small differences may be due to rounding or unconfirmed transactions)"
                );
            }
        }
        Err(e) => {
            eprintln!("  ✗ Error getting wallet balances: {}", e);
        }
    }

    println!("\n=== Lab 04 Summary ===");
    println!("✓ Listed {} UTXOs from miner wallet", utxos.len());
    println!(
        "✓ Selected spendable UTXO: {}:{}",
        selected.txid, selected.vout
    );
    println!(
        "✓ Constructed outpoint: {}:{}",
        outpoint_id.txid, outpoint_id.vout
    );
    println!("✓ Summed spendable balance: {} BTC", spendable_sum);
    println!("✓ Reconciled with Bitcoin Core wallet balance");

    println!("\n=== Key Insight ===");
    println!("A Bitcoin wallet balance is NOT an account entry stored somewhere.");
    println!("Instead, it's calculated by summing all spendable UTXOs that the wallet controls.");
    println!(
        "Each UTXO is an independent piece of Bitcoin that was received in a past transaction."
    );
    println!("When you spend, you consume entire UTXOs as inputs and create new UTXOs as outputs.");
}
