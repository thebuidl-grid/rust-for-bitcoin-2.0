//! Stretch goal: a place where raw `rust-bitcoin` beats BDK.
//!
//! BDK's `Wallet` is built around descriptors and owned UTXOs. The moment you
//! need to handle a transaction the wallet does not own, or build an output that
//! is not a payment (an `OP_RETURN` data carrier, a custom script), you drop to
//! `rust-bitcoin` and assemble `Transaction` / `TxIn` / `TxOut` by hand.
//!
//! This module does two such things with no wallet involved:
//!   1. decode an arbitrary raw transaction hex and describe it, and
//!   2. construct an `OP_RETURN` transaction skeleton from scratch.

use bdk_wallet::bitcoin::absolute::LockTime;
use bdk_wallet::bitcoin::consensus::encode::{deserialize_hex, serialize_hex};
use bdk_wallet::bitcoin::hashes::Hash;
use bdk_wallet::bitcoin::script::PushBytesBuf;
use bdk_wallet::bitcoin::{
    Amount, OutPoint, Script, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
    transaction::Version,
};

use crate::error::WalletError;

/// Decode `hex` into a `Transaction` and return a human readable description.
pub fn decode(hex: &str) -> Result<String, WalletError> {
    let tx: Transaction = deserialize_hex(hex.trim())
        .map_err(|e| WalletError::BuildTx(format!("could not decode transaction: {e}")))?;

    let mut out = String::new();
    out.push_str(&format!("txid:      {}\n", tx.compute_txid()));
    out.push_str(&format!("wtxid:     {}\n", tx.compute_wtxid()));
    out.push_str(&format!("version:   {}\n", tx.version.0));
    out.push_str(&format!("locktime:  {}\n", tx.lock_time));
    out.push_str(&format!("weight:    {} wu\n", tx.weight()));
    out.push_str(&format!("vsize:     {} vB\n", tx.vsize()));
    out.push_str(&format!(
        "segwit:    {}\n",
        tx.input.iter().any(|i| !i.witness.is_empty())
    ));

    out.push_str(&format!("\ninputs ({}):\n", tx.input.len()));
    for (i, txin) in tx.input.iter().enumerate() {
        out.push_str(&format!(
            "  [{i}] {}:{} sequence={:#010x} witness_items={}\n",
            txin.previous_output.txid,
            txin.previous_output.vout,
            txin.sequence.0,
            txin.witness.len(),
        ));
    }

    let total_out: Amount = tx.output.iter().map(|o| o.value).sum();
    out.push_str(&format!(
        "\noutputs ({}), total {}:\n",
        tx.output.len(),
        total_out
    ));
    for (i, txout) in tx.output.iter().enumerate() {
        out.push_str(&format!(
            "  [{i}] {:>15} sat  {}\n",
            txout.value.to_sat(),
            describe_script(&txout.script_pubkey),
        ));
    }

    return Ok(out);
}

/// Build an `OP_RETURN` transaction skeleton by hand and return `(txid, hex)`.
///
/// The single input is a placeholder (all zero outpoint) because the point is
/// the structure, not a spendable transaction. This is exactly the kind of
/// object BDK will not build for you.
pub fn build_op_return(message: &str) -> Result<(Txid, String), WalletError> {
    // 80 bytes is the standardness limit relays enforce for a single OP_RETURN.
    if message.len() > 80 {
        return Err(WalletError::BuildTx(format!(
            "OP_RETURN payload is {} bytes, the standard limit is 80",
            message.len()
        )));
    }

    let mut payload = PushBytesBuf::new();
    payload
        .extend_from_slice(message.as_bytes())
        .map_err(|e| WalletError::BuildTx(format!("OP_RETURN payload rejected: {e}")))?;

    let data_output = TxOut {
        value: Amount::ZERO,
        script_pubkey: ScriptBuf::new_op_return(&payload),
    };

    let placeholder_input = TxIn {
        previous_output: OutPoint::new(Txid::all_zeros(), 0),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
    };

    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![placeholder_input],
        output: vec![data_output],
    };

    return Ok((tx.compute_txid(), serialize_hex(&tx)));
}

fn describe_script(script: &Script) -> String {
    if script.is_p2pkh() {
        return "p2pkh".to_string();
    }
    if script.is_p2sh() {
        return "p2sh".to_string();
    }
    if script.is_p2wpkh() {
        return "p2wpkh".to_string();
    }
    if script.is_p2wsh() {
        return "p2wsh".to_string();
    }
    if script.is_p2tr() {
        return "p2tr".to_string();
    }
    if script.is_op_return() {
        return format!("op_return ({} bytes)", script.len().saturating_sub(2));
    }
    return "nonstandard".to_string();
}
