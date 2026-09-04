use bitcoin::consensus::encode::serialize_hex;
use bitcoin::{Address, Amount, FeeRate, Txid};
use bitcoincore_rpc::{Client, RpcApi};

use crate::error::TxError;
use crate::wallet::WalletDb;

/// Builds, signs, and broadcasts a transaction paying `amount` to
/// `recipient` at `fee_rate`, then persists the updated wallet state.
pub fn send(
    wallet: &mut bdk_wallet::PersistedWallet<WalletDb>,
    client: &Client,
    db: &mut WalletDb,
    recipient: &Address,
    amount: Amount,
    fee_rate: FeeRate,
) -> Result<Txid, TxError> {
    let mut builder = wallet.build_tx();
    builder.add_recipient(recipient.script_pubkey(), amount).fee_rate(fee_rate);
    let mut psbt = builder.finish().map_err(|e| TxError::Build(e.to_string()))?;

    let finalized = wallet
        .sign(&mut psbt, bdk_wallet::SignOptions::default())
        .map_err(|e| TxError::Build(e.to_string()))?;
    if !finalized {
        return Err(TxError::NotFullyFinalized);
    }

    // Raw rust-bitcoin excursion: BDK's TxBuilder/PSBT flow above covers
    // everything the minimum requirements need, but it doesn't surface
    // weight/vsize/raw-hex introspection directly. Once the PSBT is
    // finalized we drop down to plain rust-bitcoin types for that — the
    // kind of low-level access you'd reach for rust-bitcoin directly for,
    // rather than going through BDK.
    let tx = psbt.extract_tx().map_err(|e| TxError::Extract(e.to_string()))?;
    println!("tx txid: {}", tx.compute_txid());
    println!("tx weight: {}", tx.weight());
    println!("tx vsize: {}", tx.vsize());
    println!("tx raw hex: {}", serialize_hex(&tx));

    let txid = client.send_raw_transaction(&tx)?;

    wallet.persist(db).map_err(|e| TxError::Persistence(e.to_string()))?;

    Ok(txid)
}
