use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address};
use rfb_labs_week_1::labs::lab03_maturity::{get_balances, mine_blocks};
use rfb_labs_week_1::labs::lab05_mempool::send_btc;
use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    println!("=== Lab 06: Transaction Decoding ===\n");

    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
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

    println!("\nStep 3: Sending payment to create transaction...\n");

    let receiver_address = get_new_address(&rpc, "receiver", "payment").unwrap();
    println!("  Receiver address: {}", receiver_address);

    let txid = match send_btc(&rpc, "miner", &receiver_address, 1.5) {
        Ok(txid) => {
            println!("  ✓ Payment sent: {}", txid);
            txid
        }
        Err(e) => {
            eprintln!("  ✗ Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("  ℹ Mining block to confirm transaction...");
    mine_blocks(&rpc, &miner_address, 1).unwrap();
    println!("  ✓ Transaction confirmed");

    println!("\nStep 4: Decoding transaction...\n");

    let decoded = match decode_verbose_transaction(&rpc, &txid) {
        Ok(tx) => {
            println!("  ✓ Transaction decoded successfully");
            tx
        }
        Err(e) => {
            eprintln!("  ✗ Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("\n=== Transaction Details ===\n");
    println!("TXID:   {}", decoded.txid);
    println!("vSize:  {} bytes", decoded.vsize);

    println!("\nInputs ({}):", decoded.inputs.len());
    for (i, input) in decoded.inputs.iter().enumerate() {
        println!("  Input #{}:", i);
        println!("    Previous TXID: {}", input.previous_output.txid);
        println!("    Previous vout: {}", input.previous_output.vout);
        println!("    Value:         {} BTC", input.previous_value);
    }

    println!("\nOutputs ({}):", decoded.outputs.len());
    for (i, output) in decoded.outputs.iter().enumerate() {
        println!("  Output #{}:", i);
        println!("    vout:          {}", output.vout);
        println!("    Value:         {} BTC", output.value);
        println!(
            "    Address:       {}",
            output.address.as_deref().unwrap_or("(none)")
        );
        println!(
            "    ScriptPubKey:  {}...",
            &output.script_pub_key_hex[..20.min(output.script_pub_key_hex.len())]
        );
    }

    println!("\n=== Analysis ===\n");

    let outpoints = input_outpoints(&decoded);
    println!("Consumed Outpoints:");
    for op in &outpoints {
        println!("  {}:{}", op.txid, op.vout);
    }

    let payment_change = match identify_payment_and_change(&decoded, &receiver_address) {
        Ok(pc) => pc,
        Err(e) => {
            eprintln!("\n✗ Error identifying payment/change: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nPayment Output:");
    println!("  Value:   {} BTC", payment_change.payment.value);
    println!(
        "  Address: {}",
        payment_change
            .payment
            .address
            .as_deref()
            .unwrap_or("(none)")
    );

    if let Some(change) = &payment_change.change {
        println!("\nChange Output:");
        println!("  Value:   {} BTC", change.value);
        println!(
            "  Address: {}",
            change.address.as_deref().unwrap_or("(none)")
        );
    } else {
        println!("\nNo change output (exact payment)");
    }

    let fee = match calculate_fee(&decoded) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("\n✗ Error calculating fee: {}", e);
            std::process::exit(1);
        }
    };

    println!("\n=== Value Conservation ===\n");

    let input_sum: f64 = decoded.inputs.iter().map(|i| i.previous_value).sum();
    let output_sum: f64 = decoded.outputs.iter().map(|o| o.value).sum();

    println!("Total Input:   {} BTC", input_sum);
    println!("Total Output:  {} BTC", output_sum);
    println!("Miner Fee:     {} BTC", fee);
    println!("\nVerification: {} - {} = {}", input_sum, output_sum, fee);

    if (input_sum - output_sum - fee).abs() < 0.000_000_01 {
        println!("✓ Value is conserved: inputs = outputs + fee");
    } else {
        println!("✗ Value NOT conserved!");
    }

    println!("\n=== Lab 06 Complete ===");
}
