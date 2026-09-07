//! Building, signing and broadcasting spends.
//!
//! Coin selection is exposed rather than left to the default. BDK's default is
//! branch-and-bound with a random-draw fallback, which is the right default
//! because it tries to find a changeless solution; largest-first and
//! oldest-first are here so the difference in input count and fee is something
//! you can see for yourself on regtest.

use bdk_wallet::coin_selection::{LargestFirstCoinSelection, OldestFirstCoinSelection};
use bdk_wallet::error::CreateTxError;
use bdk_wallet::{KeychainKind, SignOptions, TxBuilder, TxOrdering};
use bitcoin::{Address, Amount, FeeRate, Psbt, Transaction};
use clap::ValueEnum;

use crate::error::{Error, Result};
use crate::wallet::WalletHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Selection {
    /// Branch and bound, falling back to a single random draw. BDK's default.
    Bnb,
    /// Spend the biggest UTXOs first: fewest inputs, worst privacy.
    LargestFirst,
    /// Spend the oldest UTXOs first: consolidates dust, ages the wallet down.
    OldestFirst,
}

impl Selection {
    pub fn as_str(self) -> &'static str {
        match self {
            Selection::Bnb => "branch-and-bound",
            Selection::LargestFirst => "largest-first",
            Selection::OldestFirst => "oldest-first",
        }
    }
}

pub struct SpendRequest {
    pub recipient: Address,
    pub amount: Amount,
    pub fee_rate: FeeRate,
    pub selection: Selection,
    /// Send everything and let the recipient absorb the fee.
    pub drain: bool,
}

#[derive(Debug)]
pub struct SpendDraft {
    pub psbt: Psbt,
    pub tx: Transaction,
    pub fee: Amount,
    pub fee_rate: FeeRate,
    pub inputs: usize,
    pub outputs: usize,
    pub change: Option<Amount>,
}

/// Build a PSBT and sign it. Nothing is broadcast here.
pub fn build_and_sign(handle: &mut WalletHandle, req: &SpendRequest) -> Result<SpendDraft> {
    handle.require_signing()?;

    if !req
        .recipient
        .as_unchecked()
        .is_valid_for_network(handle.wallet.network())
    {
        return Err(Error::AddressNetworkMismatch {
            address: req.recipient.to_string(),
            network: handle.wallet.network(),
        });
    }

    let script = req.recipient.script_pubkey();
    let wallet = &mut handle.wallet;

    // `coin_selection` changes the builder's type parameter, so each algorithm
    // needs its own arm. The shared setup lives in `configure`.
    let mut psbt = match req.selection {
        Selection::Bnb => {
            let mut builder = wallet.build_tx();
            configure(&mut builder, req, script);
            builder.finish()
        }
        Selection::LargestFirst => {
            let mut builder = wallet.build_tx().coin_selection(LargestFirstCoinSelection);
            configure(&mut builder, req, script);
            builder.finish()
        }
        Selection::OldestFirst => {
            let mut builder = wallet.build_tx().coin_selection(OldestFirstCoinSelection);
            configure(&mut builder, req, script);
            builder.finish()
        }
    }
    .map_err(describe_create_error)?;

    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
    if !finalized {
        return Err(Error::IncompleteSignature);
    }

    let fee = psbt
        .fee()
        .map_err(|e| Error::wallet("computing the transaction fee", e))?;
    let tx = psbt
        .clone()
        .extract_tx()
        .map_err(|e| Error::wallet("extracting the signed transaction", e))?;

    // Change is whichever output pays back into the internal keychain. If BDK
    // found a changeless solution there will not be one.
    let change = tx
        .output
        .iter()
        .find(|out| {
            matches!(
                wallet.derivation_of_spk(out.script_pubkey.clone()),
                Some((KeychainKind::Internal, _))
            )
        })
        .map(|out| out.value);

    let fee_rate = fee.div_by_weight_ceil(tx.weight()).unwrap_or(FeeRate::ZERO);

    Ok(SpendDraft {
        inputs: tx.input.len(),
        outputs: tx.output.len(),
        fee,
        fee_rate,
        change,
        tx,
        psbt,
    })
}

fn configure<Cs>(builder: &mut TxBuilder<'_, Cs>, req: &SpendRequest, script: bitcoin::ScriptBuf) {
    if req.drain {
        builder.drain_wallet().drain_to(script);
    } else {
        builder.add_recipient(script, req.amount);
    }
    builder.fee_rate(req.fee_rate);
    // Shuffling outputs keeps the change output from always landing in the same
    // position, which is the cheapest privacy win available here.
    builder.ordering(TxOrdering::Shuffle);
}

/// Turn BDK's "insufficient funds" into a message with the actual numbers in it.
fn describe_create_error(err: CreateTxError) -> Error {
    match err {
        CreateTxError::CoinSelection(inner) => Error::InsufficientFunds {
            available: inner.available.to_string(),
            needed: inner.needed.to_string(),
        },
        other => Error::CreateTx(other),
    }
}
