//! Independent transaction verification with plain `rust-bitcoin`.
//!
//! This is the part of the wallet that deliberately does not go through BDK.
//! BDK will happily tell you that a transaction it built is valid, but that is
//! the wallet marking its own homework. To check that a broadcast transaction is
//! what we think it is, we pull the raw bytes back from the node, decode them
//! with `bitcoin::consensus`, recompute the txid from the decoded structure,
//! rebuild the sighash for every input, and verify the signature in the witness
//! against the public key committed to by the previous output's scriptPubKey.
//!
//! None of that needs wallet state, and BDK has no API for it, because it is not
//! a wallet operation. It is consensus-level plumbing, which is exactly what
//! `rust-bitcoin` is for.

use std::collections::HashMap;

use bitcoin::consensus::encode;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{Message, Secp256k1, XOnlyPublicKey};
use bitcoin::sighash::{Prevouts, SighashCache};
use bitcoin::{Amount, OutPoint, PublicKey, Transaction, TxOut, Txid};
use bitcoincore_rpc::Client;

use crate::error::{Error, Result};
use crate::node;
use crate::wallet::WalletHandle;

/// Where the raw bytes came from. Worth reporting: verifying against the node's
/// copy is a stronger statement than verifying against our own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxSource {
    Node,
    WalletGraph,
}

impl TxSource {
    pub fn as_str(self) -> &'static str {
        match self {
            TxSource::Node => "bitcoind (getrawtransaction)",
            TxSource::WalletGraph => "local wallet transaction graph",
        }
    }
}

#[derive(Debug)]
pub struct InputReport {
    pub index: usize,
    pub outpoint: OutPoint,
    pub value: Amount,
    pub spend_type: &'static str,
    pub signature_valid: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct TxReport {
    pub source: TxSource,
    pub raw_len: usize,
    pub declared_txid: Txid,
    pub computed_txid: Txid,
    pub wtxid: bitcoin::Wtxid,
    pub version: i32,
    pub lock_time: bitcoin::absolute::LockTime,
    pub weight: bitcoin::Weight,
    pub vsize: usize,
    pub input_total: Option<Amount>,
    pub output_total: Amount,
    pub fee: Option<Amount>,
    pub inputs: Vec<InputReport>,
    pub outputs: Vec<(usize, Amount, String)>,
}

/// Decode `txid` from the node (or, failing that, from our own graph) and check
/// everything about it that can be checked without trusting the source.
pub fn verify(handle: &WalletHandle, client: Option<&Client>, txid: Txid) -> Result<TxReport> {
    let (tx, source) = fetch(handle, client, txid)?;

    // Re-serialise so the reported size describes the bytes we actually decoded.
    let raw = encode::serialize(&tx);
    let computed_txid = tx.compute_txid();

    let prevouts = collect_prevouts(handle, client, &tx)?;
    let input_total = prevouts.as_ref().map(|map| {
        tx.input
            .iter()
            .filter_map(|i| map.get(&i.previous_output))
            .map(|o| o.value)
            .sum()
    });
    let output_total: Amount = tx.output.iter().map(|o| o.value).sum();
    let fee = input_total.and_then(|total: Amount| total.checked_sub(output_total));

    let inputs = match &prevouts {
        Some(map) => verify_inputs(&tx, map)?,
        None => tx
            .input
            .iter()
            .enumerate()
            .map(|(index, txin)| InputReport {
                index,
                outpoint: txin.previous_output,
                value: Amount::ZERO,
                spend_type: "unknown",
                signature_valid: None,
                note: Some(
                    "previous output unavailable; run bitcoind with -txindex to verify signatures"
                        .to_string(),
                ),
            })
            .collect(),
    };

    let outputs = tx
        .output
        .iter()
        .enumerate()
        .map(|(i, out)| (i, out.value, describe_spk(&out.script_pubkey)))
        .collect();

    Ok(TxReport {
        source,
        raw_len: raw.len(),
        declared_txid: txid,
        computed_txid,
        wtxid: tx.compute_wtxid(),
        version: tx.version.0,
        lock_time: tx.lock_time,
        weight: tx.weight(),
        vsize: tx.vsize(),
        input_total,
        output_total,
        fee,
        inputs,
        outputs,
    })
}

fn fetch(
    handle: &WalletHandle,
    client: Option<&Client>,
    txid: Txid,
) -> Result<(Transaction, TxSource)> {
    if let Some(client) = client
        && let Some(tx) = node::raw_transaction(client, &txid)?
    {
        // Round-trip through the consensus codec rather than trusting the
        // client's own decode: this is the step being demonstrated.
        let bytes = encode::serialize(&tx);
        let decoded: Transaction = encode::deserialize(&bytes)?;
        return Ok((decoded, TxSource::Node));
    }

    if let Some(wallet_tx) = handle.wallet.get_tx(txid) {
        return Ok(((*wallet_tx.tx_node.tx).clone(), TxSource::WalletGraph));
    }

    Err(Error::TransactionNotFound(txid))
}

/// Resolve every input's previous output. Returns `None` if any of them is
/// unavailable, since a partial set cannot support a fee calculation.
fn collect_prevouts(
    handle: &WalletHandle,
    client: Option<&Client>,
    tx: &Transaction,
) -> Result<Option<HashMap<OutPoint, TxOut>>> {
    let mut map = HashMap::new();

    for txin in &tx.input {
        let op = txin.previous_output;

        if let Some(utxo) = handle.wallet.get_utxo(op) {
            map.insert(op, utxo.txout);
            continue;
        }
        if let Some(wallet_tx) = handle.wallet.get_tx(op.txid)
            && let Some(out) = wallet_tx.tx_node.tx.output.get(op.vout as usize)
        {
            map.insert(op, out.clone());
            continue;
        }
        if let Some(client) = client
            && let Some(prev) = node::raw_transaction(client, &op.txid)?
            && let Some(out) = prev.output.get(op.vout as usize)
        {
            map.insert(op, out.clone());
            continue;
        }

        return Ok(None);
    }

    Ok(Some(map))
}

/// Rebuild each input's sighash and verify the witness signature against it.
fn verify_inputs(
    tx: &Transaction,
    prevouts: &HashMap<OutPoint, TxOut>,
) -> Result<Vec<InputReport>> {
    let secp = Secp256k1::verification_only();

    // Taproot sighashes commit to every previous output, so the full ordered
    // list has to be built before any single input can be checked.
    let ordered: Vec<TxOut> = tx
        .input
        .iter()
        .map(|txin| {
            prevouts
                .get(&txin.previous_output)
                .cloned()
                .ok_or_else(|| Error::Unverifiable {
                    index: 0,
                    reason: "missing previous output".into(),
                })
        })
        .collect::<Result<_>>()?;

    let mut cache = SighashCache::new(tx);
    let mut reports = Vec::with_capacity(tx.input.len());

    for (index, txin) in tx.input.iter().enumerate() {
        let prev = &ordered[index];
        let spk = &prev.script_pubkey;

        let (spend_type, signature_valid, note) = if spk.is_p2wpkh() {
            match verify_p2wpkh(&secp, &mut cache, index, prev, &txin.witness) {
                Ok(valid) => ("p2wpkh (segwit v0 key spend)", Some(valid), None),
                Err(reason) => ("p2wpkh (segwit v0 key spend)", None, Some(reason)),
            }
        } else if spk.is_p2tr() {
            match verify_p2tr_key_spend(&secp, &mut cache, index, &ordered, spk, &txin.witness) {
                Ok(valid) => ("p2tr (taproot key spend)", Some(valid), None),
                Err(reason) => ("p2tr", None, Some(reason)),
            }
        } else {
            (
                describe_spk_static(spk),
                None,
                Some("signature verification implemented for p2wpkh and p2tr key spends".into()),
            )
        };

        reports.push(InputReport {
            index,
            outpoint: txin.previous_output,
            value: prev.value,
            spend_type,
            signature_valid,
            note,
        });
    }

    Ok(reports)
}

fn verify_p2wpkh(
    secp: &Secp256k1<bitcoin::secp256k1::VerifyOnly>,
    cache: &mut SighashCache<&Transaction>,
    index: usize,
    prev: &TxOut,
    witness: &bitcoin::Witness,
) -> std::result::Result<bool, String> {
    if witness.len() != 2 {
        return Err(format!(
            "expected a 2-element witness (signature, pubkey), found {}",
            witness.len()
        ));
    }

    let sig = bitcoin::ecdsa::Signature::from_slice(&witness[0])
        .map_err(|e| format!("malformed ECDSA signature: {e}"))?;
    let pubkey =
        PublicKey::from_slice(&witness[1]).map_err(|e| format!("malformed public key: {e}"))?;

    // The scriptPubKey commits to HASH160(pubkey); check the witness pubkey
    // really is the one the output was locked to before checking the signature.
    let expected = pubkey.wpubkey_hash().map_err(|_| {
        "witness public key is uncompressed, which is invalid under BIP143".to_string()
    })?;
    if prev.script_pubkey.as_bytes()[2..] != expected.to_byte_array() {
        return Ok(false);
    }

    let sighash_type = sig.sighash_type;
    let sighash = cache
        .p2wpkh_signature_hash(index, &prev.script_pubkey, prev.value, sighash_type)
        .map_err(|e| format!("computing the BIP143 sighash: {e}"))?;

    let message = Message::from_digest(sighash.to_byte_array());
    Ok(secp
        .verify_ecdsa(&message, &sig.signature, &pubkey.inner)
        .is_ok())
}

fn verify_p2tr_key_spend(
    secp: &Secp256k1<bitcoin::secp256k1::VerifyOnly>,
    cache: &mut SighashCache<&Transaction>,
    index: usize,
    prevouts: &[TxOut],
    spk: &bitcoin::Script,
    witness: &bitcoin::Witness,
) -> std::result::Result<bool, String> {
    if witness.len() != 1 {
        return Err(format!(
            "not a key spend: witness has {} elements",
            witness.len()
        ));
    }

    let sig = bitcoin::taproot::Signature::from_slice(&witness[0])
        .map_err(|e| format!("malformed Schnorr signature: {e}"))?;

    // The output key is the 32 bytes of the witness program itself.
    let output_key = XOnlyPublicKey::from_slice(&spk.as_bytes()[2..])
        .map_err(|e| format!("malformed taproot output key: {e}"))?;

    let sighash_type = sig.sighash_type;
    let sighash = cache
        .taproot_key_spend_signature_hash(index, &Prevouts::All(prevouts), sighash_type)
        .map_err(|e| format!("computing the BIP341 sighash: {e}"))?;

    let message = Message::from_digest(sighash.to_byte_array());
    Ok(secp
        .verify_schnorr(&sig.signature, &message, &output_key)
        .is_ok())
}

fn describe_spk(spk: &bitcoin::Script) -> String {
    describe_spk_static(spk).to_string()
}

fn describe_spk_static(spk: &bitcoin::Script) -> &'static str {
    if spk.is_p2pkh() {
        "p2pkh"
    } else if spk.is_p2sh() {
        "p2sh"
    } else if spk.is_p2wpkh() {
        "p2wpkh"
    } else if spk.is_p2wsh() {
        "p2wsh"
    } else if spk.is_p2tr() {
        "p2tr"
    } else if spk.is_op_return() {
        "op_return"
    } else {
        "non-standard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::ScriptBuf;
    use bitcoin::hex::FromHex;

    /// The two transactions from `evidence/demo-transcript.md`, captured as raw
    /// hex so the sighash and signature checks can run without a node.
    ///
    /// `SPEND_WPKH` pays a Taproot output from a native SegWit one, and
    /// `SPEND_TAPROOT` spends that Taproot output straight back with a key spend.
    /// Between them they cover both verification paths.
    const SPEND_WPKH: &str = "020000000001011bbe29df365d320d31e35604ae936c1317a8d435ef1ac38d07c526b788b62d9d0000000000fdffff\
ff0200e1f50500000000225120ad94df9b7698d9a4c58c72415367392bcf79cb1ba323a3fbd765c9c256ac6f78cf0f\
102401000000160014f2241eb520f6b23f7f7e3b5105defa9ed4000b7f02473044022062ce9d2be290d7b3ecb13d1a\
6941de73dea2319b8b89f4e04d4ab4941ad4254702200ae05f3f300a11c6caa551e63424920df8b8a9ea081ad7ceff\
a051bafb26c434012102d6a5b020f5a8b684921526e030be1ace96ea400174b841f3e17bf683f528736165000000";

    const SPEND_TAPROOT: &str = "02000000000101de2032d6b281ef385c10e050dc0478a3e5500ccf359da575e9831f819248d98b0000000000fdffff\
ff025485930300000000225120494b988fde5d5fef4a16230021634c22fc469775d1e5d7b76d663237ef082a0f005a\
620200000000160014d7b71b22b9a10a26a8fb8a7139286d7758a87a1c0140efee6b8d980c49426a2d83ebf0fcc824\
a03df46c1e242f088c941ae029ed3d1d37f845d91175902391e7b04431e2e325cb6627ea452fcf18094383c44e62cc\
dd66000000";

    /// The coinbase output that `SPEND_WPKH` spends.
    const WPKH_PREVOUT_SPK: &str = "001431892b614f34532d68181c932a6648d63bde0270";
    const WPKH_PREVOUT_SAT: u64 = 5_000_000_000;

    fn decode(hex: &str) -> Transaction {
        let bytes = Vec::<u8>::from_hex(hex).expect("hex");
        encode::deserialize::<Transaction>(&bytes).expect("consensus decode")
    }

    fn prevout_map(tx: &Transaction, outs: Vec<TxOut>) -> HashMap<OutPoint, TxOut> {
        tx.input
            .iter()
            .map(|txin| txin.previous_output)
            .zip(outs)
            .collect()
    }

    fn wpkh_prevout() -> TxOut {
        TxOut {
            value: Amount::from_sat(WPKH_PREVOUT_SAT),
            script_pubkey: ScriptBuf::from_hex(WPKH_PREVOUT_SPK).expect("spk"),
        }
    }

    #[test]
    fn recomputes_the_txid_from_the_raw_bytes() {
        let tx = decode(SPEND_WPKH);
        assert_eq!(
            tx.compute_txid().to_string(),
            "8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de"
        );
        assert_eq!(tx.weight().to_wu(), 609);
        assert_eq!(tx.vsize(), 153);
    }

    #[test]
    fn verifies_a_p2wpkh_signature() {
        let tx = decode(SPEND_WPKH);
        let prevouts = prevout_map(&tx, vec![wpkh_prevout()]);
        let reports = verify_inputs(&tx, &prevouts).expect("verify");

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].spend_type, "p2wpkh (segwit v0 key spend)");
        assert_eq!(reports[0].signature_valid, Some(true));
    }

    #[test]
    fn verifies_a_taproot_key_spend() {
        let funding = decode(SPEND_WPKH);
        let tx = decode(SPEND_TAPROOT);
        // The Taproot spend consumes output 0 of the transaction above.
        let prevouts = prevout_map(&tx, vec![funding.output[0].clone()]);
        let reports = verify_inputs(&tx, &prevouts).expect("verify");

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].spend_type, "p2tr (taproot key spend)");
        assert_eq!(reports[0].signature_valid, Some(true));
    }

    #[test]
    fn rejects_a_tampered_p2wpkh_signature() {
        let mut tx = decode(SPEND_WPKH);
        let mut witness = tx.input[0].witness.to_vec();
        // Flip a byte inside the DER signature, leaving its structure intact.
        let last = witness[0].len() - 2;
        witness[0][last] ^= 0x01;
        tx.input[0].witness = bitcoin::Witness::from_slice(&witness);

        let prevouts = prevout_map(&tx, vec![wpkh_prevout()]);
        let reports = verify_inputs(&tx, &prevouts).expect("verify");
        assert_eq!(reports[0].signature_valid, Some(false));
    }

    #[test]
    fn rejects_a_tampered_taproot_signature() {
        let funding = decode(SPEND_WPKH);
        let mut tx = decode(SPEND_TAPROOT);
        let mut witness = tx.input[0].witness.to_vec();
        witness[0][10] ^= 0xff;
        tx.input[0].witness = bitcoin::Witness::from_slice(&witness);

        let prevouts = prevout_map(&tx, vec![funding.output[0].clone()]);
        let reports = verify_inputs(&tx, &prevouts).expect("verify");
        assert_eq!(reports[0].signature_valid, Some(false));
    }

    #[test]
    fn notices_when_an_output_pays_a_different_key() {
        let tx = decode(SPEND_WPKH);
        // Same value, but locked to a different pubkey hash than the witness
        // carries. The signature would still verify against its own key, so this
        // checks the commitment test rather than the signature test.
        let mut prevout = wpkh_prevout();
        prevout.script_pubkey =
            ScriptBuf::from_hex("0014000000000000000000000000000000000000dead").expect("spk");

        let prevouts = prevout_map(&tx, vec![prevout]);
        let reports = verify_inputs(&tx, &prevouts).expect("verify");
        assert_eq!(reports[0].signature_valid, Some(false));
    }

    #[test]
    fn describes_output_types() {
        let tx = decode(SPEND_WPKH);
        assert_eq!(describe_spk(&tx.output[0].script_pubkey), "p2tr");
        assert_eq!(describe_spk(&tx.output[1].script_pubkey), "p2wpkh");
    }
}
