//! Stretch goal: a CLTV-timelocked P2WSH "vault", built and spent with raw `rust-bitcoin`.
//!
//! `bdk_wallet`'s `TxBuilder`/`Wallet::sign` only know how to satisfy scripts that come from a
//! wallet descriptor. A custom witness script -- like a single-key output that can't be spent
//! before a given block height -- isn't one of those, so there is no descriptor to derive a
//! signer from. Building the redeem script, computing its BIP143 sighash, and assembling the
//! witness stack by hand is exactly the kind of scenario raw `rust-bitcoin` is for.

use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use bdk_bitcoind_rpc::bitcoincore_rpc::{Client, RpcApi};
use bdk_wallet::bitcoin::absolute::LockTime;
use bdk_wallet::bitcoin::opcodes::all::{OP_CHECKSIG, OP_CLTV, OP_DROP};
use bdk_wallet::bitcoin::script::Builder;
use bdk_wallet::bitcoin::secp256k1::{Message, Secp256k1};
use bdk_wallet::bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bdk_wallet::bitcoin::{
    Address, Amount, Network, OutPoint, PrivateKey, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    Witness,
};

/// Build the redeem script `<unlock_height> OP_CLTV OP_DROP <pubkey> OP_CHECKSIG` and its
/// corresponding P2WSH address, using a freshly generated single-use key.
pub fn create(network: Network, unlock_height: u32) -> Result<()> {
    let secp = Secp256k1::new();
    let secret_key = bdk_wallet::bitcoin::secp256k1::SecretKey::new(
        &mut bdk_wallet::bitcoin::secp256k1::rand::thread_rng(),
    );
    let private_key = PrivateKey::new(secret_key, network);
    let public_key = private_key.public_key(&secp);

    let redeem_script = Builder::new()
        .push_int(unlock_height as i64)
        .push_opcode(OP_CLTV)
        .push_opcode(OP_DROP)
        .push_slice(public_key.inner.serialize())
        .push_opcode(OP_CHECKSIG)
        .into_script();

    let address = Address::p2wsh(&redeem_script, network);

    println!("Vault created (do not use outside regtest/test funds):");
    println!("  Unlock height:  {unlock_height}");
    println!(
        "  Private key:    {} (WIF, test-only, keep it to spend later)",
        private_key.to_wif()
    );
    println!("  Redeem script:  {}", redeem_script.to_hex_string());
    println!("  Vault address:  {address}");
    println!();
    println!("Fund it (e.g. `bitcoin-cli -regtest sendtoaddress {address} 0.001`), then once");
    println!("the chain reaches height {unlock_height}, spend it with `vault-spend`.");
    Ok(())
}

/// Manually build, sign, and broadcast a spend from a vault created by [`create`].
#[allow(clippy::too_many_arguments)]
pub fn spend(
    rpc: Arc<Client>,
    network: Network,
    outpoint: &str,
    amount_sats: u64,
    wif: &str,
    redeem_script_hex: &str,
    unlock_height: u32,
    to: &str,
) -> Result<()> {
    let secp = Secp256k1::new();
    let outpoint =
        OutPoint::from_str(outpoint).context("invalid --outpoint, expected txid:vout")?;
    let private_key = PrivateKey::from_wif(wif).context("invalid --wif")?;
    let redeem_script =
        ScriptBuf::from_hex(redeem_script_hex).context("invalid --redeem-script hex")?;
    let to_address = Address::from_str(to)
        .context("invalid --to address")?
        .require_network(network)?;
    let amount = Amount::from_sat(amount_sats);

    // A flat, demo-sized fee. A real implementation would estimate vsize x feerate.
    const FEE: Amount = Amount::from_sat(500);
    let send_amount = amount
        .checked_sub(FEE)
        .context("amount smaller than the demo fee")?;

    let mut tx = Transaction {
        version: bdk_wallet::bitcoin::transaction::Version::TWO,
        lock_time: LockTime::from_consensus(unlock_height),
        input: vec![TxIn {
            previous_output: outpoint,
            script_sig: ScriptBuf::new(),
            // Must be < 0xFFFFFFFF for nLockTime to be enforced.
            sequence: Sequence::ENABLE_LOCKTIME_NO_RBF,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: send_amount,
            script_pubkey: to_address.script_pubkey(),
        }],
    };

    let sighash = SighashCache::new(&tx).p2wsh_signature_hash(
        0,
        &redeem_script,
        amount,
        EcdsaSighashType::All,
    )?;
    let message = Message::from(sighash);
    let signature = secp.sign_ecdsa(&message, &private_key.inner);
    let sig = bdk_wallet::bitcoin::ecdsa::Signature {
        signature,
        sighash_type: EcdsaSighashType::All,
    };

    tx.input[0].witness = Witness::from_slice(&[sig.to_vec(), redeem_script.to_bytes()]);

    let txid = rpc.send_raw_transaction(&tx)?;
    println!("Broadcast vault spend: {txid}");
    println!("  From vault outpoint: {outpoint}");
    println!("  To:                  {to_address} ({send_amount})");
    Ok(())
}
