use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client};

use crate::config::Config;

pub fn connect(config: &Config) -> anyhow::Result<Client> {
    let auth = match (&config.rpc_cookie, &config.rpc_user, &config.rpc_pass) {
        (Some(path), _, _) => Auth::CookieFile(path.clone()),
        (None, Some(user), Some(pass)) => Auth::UserPass(user.clone(), pass.clone()),
        (None, None, None) => Auth::None,
        _ => anyhow::bail!("set both RPC_USER and RPC_PASS, or neither"),
    };
    Client::new(&config.rpc_url, auth).map_err(Into::into)
}
