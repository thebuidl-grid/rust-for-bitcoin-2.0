use rfb_labs_week_1::labs::lab01_network::inspect_network;
use rfb_labs_week_1::rpc::ProcessRpc;
use std::env;

fn main() {
    let container = env::var("RPC_DOCKER_CONTAINER").unwrap_or_else(|_| "polar-n2-backend1".into());
    let user = env::var("RPC_USER").expect("RPC_USER must be set");
    let password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD must be set");


    let rpc = ProcessRpc::new("docker").with_base_args([
        "exec".to_string(),
        container,
        "bitcoin-cli".to_string(),
        "-regtest".to_string(),
        format!("-rpcuser={user}"),
        format!("-rpcpassword={password}"),
    ]);

    match inspect_network(&rpc) {
        Ok(snapshot) => println!("{snapshot:#?}"),
        Err(error) => eprintln!("error: {error}"),
    }
}