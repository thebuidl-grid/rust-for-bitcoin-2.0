//! Lab 02 — construct and explain legacy P2PKH.

use bitcoin::hashes::Hash;
use bitcoin::script::Builder;
use bitcoin::Network;
use bitcoin::PublicKey;
use std::str::FromStr;

use crate::model::P2pkhSpendTemplate;
use crate::LabResult;

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let pubkey = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::LabError::InvalidKey(e.to_string()))?;

    let address = bitcoin::Address::p2pkh(&pubkey, network);
    Ok(address.to_string())
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let pubkey = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::LabError::InvalidKey(e.to_string()))?;

    let hash = pubkey.pubkey_hash();
    let script = Builder::new()
        .push_opcode(bitcoin::opcodes::all::OP_DUP)
        .push_opcode(bitcoin::opcodes::all::OP_HASH160)
        .push_slice(hash.as_byte_array())
        .push_opcode(bitcoin::opcodes::all::OP_EQUALVERIFY)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKSIG)
        .into_script();

    Ok(script.to_hex_string())
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    let pubkey = PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::LabError::InvalidKey(e.to_string()))?;

    Ok(pubkey.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    Ok(P2pkhSpendTemplate {
        script_sig_items: vec![signature_hex.to_string(), public_key_hex.to_string()],
        witness_items: vec![],
    })
}
