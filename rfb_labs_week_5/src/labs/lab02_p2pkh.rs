//! Lab 02 — construct and explain legacy P2PKH.

use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::model::P2pkhSpendTemplate;
use crate::{LabError, LabResult};

fn parse_public_key(public_key_hex: &str) -> LabResult<PublicKey> {
    public_key_hex
        .parse::<PublicKey>()
        .map_err(|error| LabError::InvalidKey(error.to_string()))
}

/// Derive a P2PKH address from a serialized public key.
pub fn derive_p2pkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;
    Ok(Address::p2pkh(public_key, network).to_string())
}

/// Build the P2PKH scriptPubKey for the serialized public key:
/// `OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`.
pub fn build_p2pkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;
    let script = ScriptBuf::new_p2pkh(&public_key.pubkey_hash());
    Ok(script.to_hex_string())
}

/// Return the HASH160 commitment made to the public key.
///
/// This is the *key identity* P2PKH commits to. Spend authorization (proving control of
/// the matching private key) is a separate step satisfied later in ScriptSig.
pub fn committed_pubkey_hash(public_key_hex: &str) -> LabResult<String> {
    let public_key = parse_public_key(public_key_hex)?;
    Ok(public_key.pubkey_hash().to_string())
}

/// Place a signature and public key in the legacy unlocking location.
///
/// Legacy P2PKH puts both unlocking items in ScriptSig; SegWit moves them into the
/// witness instead (see Lab 04), leaving ScriptSig empty.
pub fn p2pkh_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<P2pkhSpendTemplate> {
    // Validate the public key is well formed before it is placed in the unlocking script.
    parse_public_key(public_key_hex)?;

    Ok(P2pkhSpendTemplate {
        script_sig_items: vec![signature_hex.to_owned(), public_key_hex.to_owned()],
        witness_items: Vec::new(),
    })
}
