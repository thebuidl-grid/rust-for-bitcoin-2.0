//! Building, signing and broadcasting transactions.
//!
//! # The three stages
//!
//! ```text
//! build   TxBuilder -> coin selection -> PSBT (unsigned)
//! sign    wallet.sign(&mut psbt)      -> PSBT (finalised)
//! extract psbt.extract_tx()           -> Transaction
//! send    client.send_raw_transaction -> Txid
//! ```
//!
//! A PSBT (BIP174) is the intermediate form. It carries the unsigned transaction
//! plus everything a signer needs to know — previous outputs, derivation paths, key
//! origins — which is what lets an offline or hardware signer work from it. Here
//! both roles live in one process, but the shape is the same.
//!
//! # The trap
//!
//! [`bdk_wallet::Wallet::sign`] returns `Result<bool, SignerError>`. An `Ok(false)`
//! is **not** an error: it means "no failure occurred, but the PSBT still is not
//! fully signed". A wallet loaded without `extract_keys()` holds public-only
//! descriptors, so it produces exactly that — and if you ignore the boolean you
//! broadcast a transaction that cannot possibly confirm. We turn it into
//! [`WalletError::IncompleteSignature`].

use bdk_bitcoind_rpc::bitcoincore_rpc::{Client, RpcApi};
use bdk_wallet::bitcoin::{Address, Amount, FeeRate, Psbt, Transaction, Txid};
use bdk_wallet::{KeychainKind, SignOptions};
use bdk_wallet::coin_selection::LargestFirstCoinSelection;

use crate::error::{Result, WalletError};
use crate::wallet::Wallet;

/// Which coin selection algorithm to use.
///
/// BDK's default is branch-and-bound with a single-random-draw fallback: it tries
/// to find a combination that avoids a change output entirely, which is both
/// cheaper and better for privacy. `LargestFirst` is the naive alternative, kept
/// here so the two can be compared — it spends big UTXOs first, so it needs fewer
/// inputs but consolidates the wallet and almost always creates change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection {
    /// Branch and bound (BDK default).
    #[default]
    Default,
    /// Largest UTXOs first.
    LargestFirst,
}

/// What a build produced, before broadcast.
#[derive(Debug)]
pub struct Draft {
    pub psbt: Psbt,
    pub tx: Transaction,
    pub txid: Txid,
    pub fee: Amount,
    pub inputs: usize,
    pub outputs: usize,
    /// Index of the change output within `tx.output`, if there is one.
    pub change_vout: Option<u32>,
    pub change_amount: Amount,
}

/// Parse a recipient address and check it belongs to the wallet's network.
///
/// `Address::from_str` yields a `NetworkUnchecked` address on purpose: rust-bitcoin
/// makes you state which network you expect before you can use it, so a mainnet
/// address cannot silently be paid from a regtest wallet.
pub fn parse_address(wallet: &Wallet, raw: &str) -> Result<Address> {
    let unchecked: Address<bdk_wallet::bitcoin::address::NetworkUnchecked> = raw.trim().parse()?;
    Ok(unchecked.require_network(wallet.network())?)
}

/// Build and sign a payment, leaving it ready to broadcast.
///
/// The wallet is persisted before returning. Building reveals a change address, and
/// a revealed-but-unpersisted change address is the classic way to reuse one after
/// a crash.
pub fn build_and_sign(
    wallet: &mut Wallet,
    recipient: &Address,
    amount: Amount,
    fee_rate: FeeRate,
    selection: Selection,
) -> Result<Draft> {
    let mut psbt = {
        let w = wallet.inner_mut();

        // `TxBuilder` borrows the wallet mutably, so the two arms are built and
        // finished separately rather than sharing a variable — `coin_selection`
        // changes the builder's type parameter.
        match selection {
            Selection::Default => {
                let mut builder = w.build_tx();
                builder
                    .add_recipient(recipient.script_pubkey(), amount)
                    .fee_rate(fee_rate);
                builder.finish()?
            }
            Selection::LargestFirst => {
                let mut builder = w.build_tx().coin_selection(LargestFirstCoinSelection);
                builder
                    .add_recipient(recipient.script_pubkey(), amount)
                    .fee_rate(fee_rate);
                builder.finish()?
            }
        }
    };

    // Fee must be read from the PSBT while its input UTXOs are still attached.
    let fee = psbt.fee().unwrap_or(Amount::ZERO);

    let finalized = wallet.inner().sign(&mut psbt, SignOptions::default())?;
    if !finalized {
        return Err(WalletError::IncompleteSignature);
    }

    let tx = psbt.clone().extract_tx()?;
    let txid = tx.compute_txid();

    // Identify the change output: the one paying a script this wallet owns on the
    // internal keychain. Proof the two keychains are wired up correctly.
    let mut change_vout = None;
    let mut change_amount = Amount::ZERO;
    for (vout, out) in tx.output.iter().enumerate() {
        if let Some((KeychainKind::Internal, _)) =
            wallet.inner().derivation_of_spk(out.script_pubkey.clone())
        {
            change_vout = Some(vout as u32);
            change_amount = out.value;
        }
    }

    // Persist: the build revealed a change address, and losing that means handing
    // out the same change address twice.
    wallet.persist()?;

    Ok(Draft {
        inputs: tx.input.len(),
        outputs: tx.output.len(),
        psbt,
        tx,
        txid,
        fee,
        change_vout,
        change_amount,
    })
}

/// Relay a signed transaction through the node.
pub fn broadcast(client: &Client, tx: &Transaction) -> Result<Txid> {
    Ok(client.send_raw_transaction(tx)?)
}

/// Turn a sat/vB figure into a `FeeRate`, rejecting zero.
///
/// A zero-fee transaction is relayable on regtest but will not be mined on any real
/// network, so refusing it here avoids a confusing "broadcast succeeded, never
/// confirms" outcome.
pub fn fee_rate_from_sat_per_vb(sat_per_vb: u64) -> Result<FeeRate> {
    if sat_per_vb == 0 {
        return Err(WalletError::FeeRate(sat_per_vb));
    }
    FeeRate::from_sat_per_vb(sat_per_vb).ok_or(WalletError::FeeRate(sat_per_vb))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_zero_fee_rate() {
        assert!(matches!(
            fee_rate_from_sat_per_vb(0),
            Err(WalletError::FeeRate(0))
        ));
    }

    #[test]
    fn converts_sat_per_vb_to_sat_per_kwu() {
        // 1 vB == 4 weight units, so 1 sat/vB == 250 sat/kwu.
        assert_eq!(fee_rate_from_sat_per_vb(1).unwrap().to_sat_per_kwu(), 250);
        assert_eq!(fee_rate_from_sat_per_vb(8).unwrap().to_sat_per_kwu(), 2000);
    }

    #[test]
    fn rejects_an_overflowing_fee_rate() {
        assert!(fee_rate_from_sat_per_vb(u64::MAX).is_err());
    }
}
