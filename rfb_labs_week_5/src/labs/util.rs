//! Shared helper used by the BIP32/BIP44/BIP49/BIP84 labs.
//! Not part of the public lab API — the four `todo!()` functions per file are.

use bip39::{Language, Mnemonic};

use crate::error::LabError;
use crate::LabResult;

/// Validate a BIP39 mnemonic and derive its 512-bit seed under an optional passphrase.
pub(crate) fn seed_bytes(mnemonic: &str, passphrase: &str) -> LabResult<[u8; 64]> {
    let parsed = Mnemonic::parse_in_normalized(Language::English, mnemonic)
        .map_err(|error| LabError::InvalidMnemonic(error.to_string()))?;
    Ok(parsed.to_seed(passphrase))
}
