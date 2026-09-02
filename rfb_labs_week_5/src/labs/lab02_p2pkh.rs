//! Lab 02 — construct and explain legacy P2PKH.

use bitcoin::Network;

use crate::model::P2pkhSpendTemplate;
use crate::LabResult;

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<bitcoin::PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(bitcoin::Address::p2pkh(public_key, network).to_string())
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<bitcoin::PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    let public_key_hash = public_key.pubkey_hash();
    let script_pubkey = bitcoin::ScriptBuf::new_p2pkh(&public_key_hash);

    Ok(script_pubkey.to_hex_string())
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    let public_key = public_key_hex
        .parse::<bitcoin::PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(public_key.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    let public_key = public_key_hex
        .parse::<bitcoin::PublicKey>()
        .map_err(|error| crate::LabError::InvalidKey(error.to_string()))?;

    Ok(P2pkhSpendTemplate {
        script_sig_items: vec![signature_hex.to_owned(), public_key.to_string()],
        witness_items: Vec::new(),
    })
}
