use rfb_labs_week_1::labs::lab02_wallets::{
    address_belongs_to_wallet, create_wallet, get_new_address, list_wallets,
};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    println!("=== Lab 02: Wallets and Addresses ===\n");

    // Create RPC client pointing to Polar's regtest node
    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Step 1: Creating wallets...");

    // Create two wallets: miner and receiver
    match create_wallet(&rpc, "miner") {
        Ok(_) => println!("  ✓ Created wallet: miner"),
        Err(e) => {
            if e.to_string().contains("already exists") {
                println!("  ℹ Wallet 'miner' already exists (using existing)");
            } else {
                eprintln!("  ✗ Error creating miner wallet: {}", e);
                std::process::exit(1);
            }
        }
    }

    match create_wallet(&rpc, "receiver") {
        Ok(_) => println!("  ✓ Created wallet: receiver"),
        Err(e) => {
            if e.to_string().contains("already exists") {
                println!("  ℹ Wallet 'receiver' already exists (using existing)");
            } else {
                eprintln!("  ✗ Error creating receiver wallet: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("\nStep 2: Listing loaded wallets...");
    match list_wallets(&rpc) {
        Ok(wallets) => {
            println!("  ✓ Loaded wallets: {:?}", wallets);
            if wallets.contains(&"miner".to_string()) && wallets.contains(&"receiver".to_string()) {
                println!("  ✓ Both 'miner' and 'receiver' wallets are loaded");
            }
        }
        Err(e) => {
            eprintln!("  ✗ Error listing wallets: {}", e);
            std::process::exit(1);
        }
    }

    println!("\nStep 3: Generating addresses...");

    // Generate address for miner wallet with label "mining"
    let miner_address = match get_new_address(&rpc, "miner", "mining") {
        Ok(addr) => {
            println!("  ✓ Miner address (label: mining): {}", addr);
            if addr.starts_with("bcrt1") {
                println!("    ✓ Address uses bcrt1... regtest prefix");
            } else {
                eprintln!("    ✗ Address does NOT use bcrt1 prefix!");
            }
            addr
        }
        Err(e) => {
            eprintln!("  ✗ Error generating miner address: {}", e);
            std::process::exit(1);
        }
    };

    // Generate address for receiver wallet with label "classmate"
    let receiver_address = match get_new_address(&rpc, "receiver", "classmate") {
        Ok(addr) => {
            println!("  ✓ Receiver address (label: classmate): {}", addr);
            if addr.starts_with("bcrt1") {
                println!("    ✓ Address uses bcrt1... regtest prefix");
            } else {
                eprintln!("    ✗ Address does NOT use bcrt1 prefix!");
            }
            addr
        }
        Err(e) => {
            eprintln!("  ✗ Error generating receiver address: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nStep 4: Verifying address ownership...");

    // Check that miner wallet owns its address
    match address_belongs_to_wallet(&rpc, "miner", &miner_address) {
        Ok(true) => println!("  ✓ Miner wallet owns address: {}", miner_address),
        Ok(false) => {
            eprintln!("  ✗ Miner wallet does NOT own its own address!");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  ✗ Error checking miner address ownership: {}", e);
            std::process::exit(1);
        }
    }

    // Check that receiver wallet owns its address
    match address_belongs_to_wallet(&rpc, "receiver", &receiver_address) {
        Ok(true) => println!("  ✓ Receiver wallet owns address: {}", receiver_address),
        Ok(false) => {
            eprintln!("  ✗ Receiver wallet does NOT own its own address!");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  ✗ Error checking receiver address ownership: {}", e);
            std::process::exit(1);
        }
    }

    // Verify cross-wallet check (miner wallet should NOT own receiver's address)
    match address_belongs_to_wallet(&rpc, "miner", &receiver_address) {
        Ok(false) => {
            println!("  ✓ Correctly verified: Miner wallet does NOT own receiver's address")
        }
        Ok(true) => {
            eprintln!("  ✗ Error: Miner wallet claims to own receiver's address!");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  ✗ Error checking cross-wallet ownership: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n=== Lab 02 Summary ===");
    println!("✓ Created/loaded 2 wallets: miner, receiver");
    println!("✓ Listed all loaded wallets");
    println!("✓ Generated labeled addresses in each wallet");
    println!("✓ Verified address ownership correctly");
    println!("\n=== All Lab 02 functions working correctly! ===");
}
