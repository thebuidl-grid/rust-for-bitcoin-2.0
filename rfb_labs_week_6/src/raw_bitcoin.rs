use anyhow::{Context, Result};
use bdk_wallet::bitcoin::blockdata::opcodes;
use bdk_wallet::bitcoin::blockdata::script::Builder;
use bdk_wallet::bitcoin::psbt::Psbt;
use bdk_wallet::bitcoin::script::PushBytesBuf;
use bdk_wallet::bitcoin::{Amount, TxOut};

/// Demonstrates using raw `rust-bitcoin` to construct custom scripts and outputs.
pub struct RawBitcoinDemo;

impl RawBitcoinDemo {
    /// Builds an OP_RETURN payload script output for embedding arbitrary data/anchors on-chain.
    /// 
    /// Scenario: BDK manages standard wallet payments, but embedding custom protocol metadata
    /// (e.g. proof of existence, timestamping, or protocol state) requires direct script construction
    /// using `rust-bitcoin`.
    pub fn build_op_return_output(message: &[u8]) -> Result<TxOut> {
        let mut push_bytes = PushBytesBuf::new();
        push_bytes.extend_from_slice(message)
            .context("Failed to format message as PushBytes")?;

        let script = Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(push_bytes)
            .into_script();

        Ok(TxOut {
            value: Amount::ZERO,
            script_pubkey: script,
        })
    }

    /// Inspects and prints details of a raw PSBT using `rust-bitcoin`.
    #[allow(dead_code)]
    pub fn inspect_psbt(psbt_bytes: &[u8]) -> Result<String> {
        let psbt = Psbt::deserialize(psbt_bytes)?;
        let mut summary = String::new();
        
        summary.push_str(&format!("PSBT Version: {}\n", psbt.unsigned_tx.version));
        summary.push_str(&format!("Inputs Count: {}\n", psbt.inputs.len()));
        summary.push_str(&format!("Outputs Count: {}\n", psbt.outputs.len()));
        summary.push_str(&format!("Locktime: {}\n", psbt.unsigned_tx.lock_time));

        for (idx, input) in psbt.inputs.iter().enumerate() {
            let has_utxo = input.witness_utxo.is_some() || input.non_witness_utxo.is_some();
            summary.push_str(&format!("  Input #{}: Has UTXO Info: {}\n", idx, has_utxo));
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_return_script() {
        let msg = b"RFB Week 6 Bitcoin Wallet Assignment";
        let txout = RawBitcoinDemo::build_op_return_output(msg).unwrap();
        assert_eq!(txout.value, Amount::ZERO);
        assert!(txout.script_pubkey.is_op_return());
    }
}
