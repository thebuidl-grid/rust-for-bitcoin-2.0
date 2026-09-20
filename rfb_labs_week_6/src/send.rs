use anyhow::{Context, Result};
use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client, RpcApi};
use bdk_wallet::bitcoin::{Address, Amount};
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{PersistedWallet, SignOptions};

use crate::config::Config;

/// Build, sign, and broadcast a transaction paying `amount` to `to_address`,
/// with change (if any) automatically returned to the wallet's internal
/// (change) keychain.
pub fn send(
    wallet: &mut PersistedWallet<Connection>,
    db: &mut Connection,
    config: &Config,
    to_address: &Address,
    amount: Amount,
) -> Result<bdk_wallet::bitcoin::Txid> {
    // Build: BDK selects UTXOs from the confirmed balance, adds a change
    // output back to the internal keychain if needed, and estimates a fee.
    let mut builder = wallet.build_tx();
    builder.add_recipient(to_address.script_pubkey(), amount);
    let mut psbt = builder.finish().context("failed to build transaction")?;

    // Sign: since this wallet holds the private key (it was created from
    // our own descriptor, not just an xpub), BDK can sign locally without
    // any external signer.
    let finalized = wallet
        .sign(&mut psbt, SignOptions::default())
        .context("failed to sign transaction")?;
    anyhow::ensure!(finalized, "transaction signing did not finalize the PSBT");

    wallet
        .persist(db)
        .context("failed to persist wallet after signing")?;

    let tx = psbt
        .extract_tx()
        .context("failed to extract signed transaction")?;

    // Broadcast: hand the fully signed transaction to Bitcoin Core.
    let rpc_client = Client::new(
        &config.rpc_url,
        Auth::UserPass(config.rpc_user.clone(), config.rpc_password.clone()),
    )
    .context("failed to build RPC client for broadcasting")?;

    let txid = rpc_client
        .send_raw_transaction(&tx)
        .context("failed to broadcast transaction")?;

    Ok(txid)
}
