use rfb_labs_week_1::labs::lab01_network::inspect_network;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    println!("=== Lab 01: Regtest Network Inspection ===\n");

    // Create RPC client pointing to Polar's regtest node
    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("Connecting to Bitcoin Core node...\n");

    match inspect_network(&rpc) {
        Ok(snapshot) => {
            println!("✓ Successfully connected to Bitcoin Core node!");
            println!("✓ Network verification passed!\n");
            println!("Network Snapshot:");
            println!("  Chain:           {}", snapshot.chain);
            println!("  Block Height:    {}", snapshot.block_height);
            println!("  Best Block Hash: {}", snapshot.best_block_hash);
            println!("\n=== All Lab 01 functions working correctly! ===");
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            eprintln!("\nMake sure:");
            eprintln!("  1. Polar network is running");
            eprintln!("  2. Bitcoin Core node is active (green)");
            eprintln!("  3. bitcoin-cli is in your PATH");
            std::process::exit(1);
        }
    }
}
