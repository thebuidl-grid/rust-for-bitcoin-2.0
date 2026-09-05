//! Lab 02 — construct and explain legacy P2PKH.

use bitcoin::blockdata::opcodes::all::{OP_CHECKSIG, OP_DUP, OP_EQUALVERIFY, OP_HASH160};
use bitcoin::script::Builder;
use bitcoin::{Address, Network, PublicKey};

use crate::model::P2pkhSpendTemplate;
use crate::LabResult;

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(Address::p2pkh(public_key, network).to_string())
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let pubkey_hash = public_key.pubkey_hash();

    let script = Builder::new()
        .push_opcode(OP_DUP)
        .push_opcode(OP_HASH160)
        .push_slice(pubkey_hash)
        .push_opcode(OP_EQUALVERIFY)
        .push_opcode(OP_CHECKSIG)
        .into_script();

    Ok(script.to_hex_string())
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(public_key.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    if signature_hex.is_empty() {
        return Err(crate::LabError::InvalidScript(
            "signature cannot be empty".to_string(),
        ));
    }

    public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(P2pkhSpendTemplate {
        script_sig_items: vec![signature_hex.to_owned(), public_key_hex.to_owned()],
        witness_items: Vec::new(),
    })
}
