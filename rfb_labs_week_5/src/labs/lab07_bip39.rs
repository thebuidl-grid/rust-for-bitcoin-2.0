//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::LabResult;

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let parsed = bip39::Mnemonic::parse(mnemonic)
        .map_err(|e| crate::error::LabError::InvalidMnemonic(e.to_string()))?;
    let entropy = parsed.to_entropy();
    let word_count = parsed.word_count();
    let entropy_bits = entropy.len() * 8;
    let checksum_bits = word_count / 3;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    let parsed = bip39::Mnemonic::parse(mnemonic)
        .map_err(|e| crate::error::LabError::InvalidMnemonic(e.to_string()))?;
    let seed = parsed.to_seed(passphrase);
    Ok(hex::encode(seed))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
pub fn compare_passphrases(
    mnemonic: &str,
    protected_passphrase: &str,
) -> LabResult<PassphraseComparison> {
    let empty_passphrase_seed_hex = mnemonic_seed_hex(mnemonic, "")?;
    let protected_seed_hex = mnemonic_seed_hex(mnemonic, protected_passphrase)?;
    let seeds_differ = empty_passphrase_seed_hex != protected_seed_hex;

    Ok(PassphraseComparison {
        empty_passphrase_seed_hex,
        protected_seed_hex,
        seeds_differ,
    })
}

/// Recognize the public BIP39 test mnemonic used in the class labs.
pub fn is_public_test_mnemonic(mnemonic: &str) -> bool {
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    if words.len() != 12 {
        return false;
    }
    for i in 0..11 {
        if words[i] != "abandon" {
            return false;
        }
    }
    words[11] == "about"
}

