use rfb_labs_week_1::labs::lab01_network::{get_best_block_hash, get_block_height, get_chain, inspect_network};
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ProcessRpc::new("docker").with_base_args([
        "exec",
        "polar-n1-backend1",
        "bitcoin-cli",
        "-regtest",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);
 
    let get_chain = get_chain(&client)?;
    println!("get_chain: {}", get_chain);

    let get_block_height = get_block_height(&client)?;
    println!("get_block_height: {}", get_block_height);

    let get_best_block_hash = get_best_block_hash(&client)?;
    println!("get_best_block_hash: {}", get_best_block_hash);

    let inspect_network = inspect_network(&client)?;
    println!("inspect_network: {:?}", inspect_network);

    Ok(())

    
}