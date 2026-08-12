use rfb_labs_week_1::rpc::ProcessRpc;
use rfb_labs_week_1::labs::lab01_network;

fn main() {
    println!("--- Connecting to Polar via Docker ---");

    // Replace 'polar-app-alice' with your Polar Bitcoin Core container name/ID
    let rpc = ProcessRpc::new("docker")
        .with_base_args([
            "exec",
            "polar-n2-backend1", // Check your container name in Docker Desktop or Polar
            "bitcoin-cli",
            "-regtest",
            "-rpcuser=polaruser",
            "-rpcpassword=polarpass",
        ]);

    match lab01_network::inspect_network(&rpc) {
        Ok(info) => println!("Success!\n{:#?}", info),
        Err(e) => eprintln!("Execution Error: {:?}", e),
    }
}