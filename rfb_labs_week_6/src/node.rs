use anyhow::Result;
use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client};

use crate::config::Cli;

/// Build a `bitcoincore-rpc` client from the CLI/`.env` configuration.
///
/// Prefers a cookie file if one is set, falls back to user/pass, and finally to no auth
/// (useful for a node started with `-rpcauth`-free `-regtest -server`).
pub fn build_rpc_client(cli: &Cli) -> Result<Client> {
    let auth = match (&cli.rpc_cookie, &cli.rpc_user, &cli.rpc_pass) {
        (Some(cookie), _, _) => Auth::CookieFile(cookie.clone()),
        (None, Some(user), Some(pass)) => Auth::UserPass(user.clone(), pass.clone()),
        (None, None, None) => Auth::None,
        (None, Some(_), None) => anyhow::bail!("RPC_USER set without RPC_PASS"),
        (None, None, Some(_)) => anyhow::bail!("RPC_PASS set without RPC_USER"),
    };
    let url = format!("http://{}", cli.rpc_url.trim_start_matches("http://"));
    Ok(Client::new(&url, auth)?)
}
