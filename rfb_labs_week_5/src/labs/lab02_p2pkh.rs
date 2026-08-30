//! Lab 02 — construct and explain legacy P2PKH.

use bitcoin::Network;

use crate::model::P2pkhSpendTemplate;
use crate::LabResult;

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    use std::str::FromStr;
    let pubkey = bitcoin::PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    Ok(bitcoin::Address::p2pkh(pubkey, network).to_string())
}

/// Build the P2PKH scriptPubKey for the serialized public key.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    use std::str::FromStr;
    let pubkey = bitcoin::PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    Ok(bitcoin::Address::p2pkh(pubkey, bitcoin::Network::Bitcoin)
        .script_pubkey()
        .to_hex_string())
}

/// Return the HASH160 commitment made to the public key.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    use std::str::FromStr;
    let pubkey = bitcoin::PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    Ok(pubkey.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    use std::str::FromStr;
    // Validate key
    let _ = bitcoin::PublicKey::from_str(public_key_hex)
        .map_err(|e| crate::error::LabError::InvalidKey(e.to_string()))?;
    Ok(P2pkhSpendTemplate {
        script_sig_items: vec![signature_hex.to_owned(), public_key_hex.to_owned()],
        witness_items: vec![],
    })
}

