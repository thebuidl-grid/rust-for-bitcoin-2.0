//! Runs the Lab 01 functions against a real Polar/regtest node.
//!
//! `bitcoin-cli` isn't installed on the host, only inside Polar's Docker
//! container, so this routes calls through `docker exec` as the `bitcoin`
//! user (matching Polar's own terminal) instead of calling `bitcoin-cli`
//! directly.
//!
//! Run with: cargo run --example lab01

use rfb_labs_week_1::labs::lab01_network::inspect_network;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let client = ProcessRpc::new("docker").with_base_args([
        "exec",
        "-u",
        "bitcoin",
        "polar-n1-backend1",
        "bitcoin-cli",
        "-regtest",
    ]);

    match inspect_network(&client) {
        Ok(snapshot) => println!("{snapshot:#?}"),
        Err(error) => eprintln!("inspect_network failed: {error}"),
    }
}
