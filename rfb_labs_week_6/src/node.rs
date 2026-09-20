use anyhow::{Context, Result};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::config::Config;

/// Build an RPC client pointed at the configured Bitcoin Core node.
///
/// This only constructs the client — it doesn't make any network call yet,
/// so a wrong URL/credentials won't surface here. Use `check_connection`
/// right after this to actually verify the node is reachable.
pub fn connect(config: &Config) -> Result<Client> {
    let auth = Auth::UserPass(config.rpc_user.clone(), config.rpc_password.clone());
    let client = Client::new(&config.rpc_url, auth)
        .context("failed to construct Bitcoin Core RPC client")?;
    Ok(client)
}

/// Make one real RPC call to prove the connection actually works end-to-end
/// (reachable host, correct port, valid credentials, node fully started).
/// This is a cheap sanity check to run once at startup, before trusting the
/// client for anything wallet-related.
pub fn check_connection(client: &Client) -> Result<()> {
    let info = client
        .get_blockchain_info()
        .context("failed to reach Bitcoin Core via RPC")?;

    println!(
        "Connected to Bitcoin Core: chain={}, blocks={}, headers={}",
        info.chain, info.blocks, info.headers
    );
    Ok(())
}
