//! Connects to a Bitcoin Core node over RPC (`bitcoincore-rpc`).

use anyhow::{Context, Result};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::config::Config;

pub fn connect(config: &Config) -> Result<Client> {
    let client = Client::new(
        &config.rpc_url,
        Auth::UserPass(config.rpc_user.clone(), config.rpc_pass.clone()),
    )
    .context("failed to build the Bitcoin Core RPC client")?;

    let info = client
        .get_blockchain_info()
        .context("failed to reach the Bitcoin Core node -- is it running and reachable?")?;
    println!(
        "Connected to node: chain={}, blocks={}, headers={}",
        info.chain, info.blocks, info.headers
    );

    Ok(client)
}
