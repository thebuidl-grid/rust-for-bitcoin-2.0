use rfb_labs_week_1::labs::lab01_network::inspect_network;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let rpc = ProcessRpc::new("docker").with_base_args([
        "exec",
        "polar-n3-backend1",
        "bitcoin-cli",
        "-regtest",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    match inspect_network(&rpc) {
        Ok(snapshot) => println!("{snapshot:#?}"),
        Err(error) => eprintln!("failed: {error}"),
    }
}
