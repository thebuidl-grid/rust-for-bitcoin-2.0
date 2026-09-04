use bitcoin::address::NetworkChecked;
use bitcoin::{Address, BlockHash};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::config::RpcAuthConfig;
use crate::error::NodeError;

/// Number of blocks needed for a coinbase output to mature and become
/// spendable on regtest.
pub const COINBASE_MATURITY: u64 = 100;

fn to_rpc_auth(auth: &RpcAuthConfig) -> Auth {
    match auth {
        RpcAuthConfig::CookieFile(path) => Auth::CookieFile(path.clone()),
        RpcAuthConfig::UserPass(user, pass) => Auth::UserPass(user.clone(), pass.clone()),
        RpcAuthConfig::None => Auth::None,
    }
}

/// Connects to a Bitcoin Core node over RPC using bitcoincore-rpc directly.
/// Used for regtest-only chain control (mining, direct broadcast) that
/// isn't a wallet operation — the wallet's own sync goes through
/// `bdk_bitcoind_rpc::Emitter` instead (see `sync_wallet`).
pub fn build_rpc_client(rpc_url: &str, auth: &RpcAuthConfig) -> Result<Client, NodeError> {
    Client::new(rpc_url, to_rpc_auth(auth)).map_err(|e| NodeError::ClientBuild(e.to_string()))
}

/// Confirms the node is reachable and reports which chain it's on.
pub fn chain_info(client: &Client) -> Result<(String, u64), NodeError> {
    let info = client.get_blockchain_info()?;
    Ok((info.chain.to_string(), info.blocks))
}

/// Mines `blocks` regtest blocks paying the coinbase to `address`.
/// Regtest-only: real chains don't let you mine on demand. Coinbase
/// outputs need `COINBASE_MATURITY` further confirmations before
/// they're spendable, so mine `COINBASE_MATURITY + 1` (or more) if the
/// goal is a spendable balance rather than just moving the tip forward.
pub fn fund_wallet_regtest(
    client: &Client,
    address: &Address<NetworkChecked>,
    blocks: u64,
) -> Result<Vec<BlockHash>, NodeError> {
    Ok(client.generate_to_address(blocks, address)?)
}

// TODO(stage 8): sync_wallet()
