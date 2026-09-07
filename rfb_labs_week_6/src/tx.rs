use bdk_wallet::bitcoin::{Address, Amount, Txid};
use bdk_wallet::bitcoin::address::NetworkChecked;
use bdk_wallet::SignOptions;
use anyhow::Result;
use crate::bitrpc::BitRpcClient;
use crate::wallet::SqliteWallet;

pub fn send(
    wallet: &mut SqliteWallet,
    client: &BitRpcClient,
    dest: &Address<NetworkChecked>,
    amount: Amount
) -> Result<Txid> {
    let mut builder = wallet.build_tx();
    builder.add_recipient(dest.script_pubkey(), amount);
    let mut psbt = builder.finish()?;

    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;

    if !finalized {
        anyhow::bail!("failed to finalize PSBT -- missing signatures?");
    };

    let tx = psbt.extract_tx()?;
    let txid = client.send_raw_txn(&tx)?;

    Ok(txid)
}