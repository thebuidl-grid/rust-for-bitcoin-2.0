//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use bip39::{Language, Mnemonic};

use crate::error::LabError;
use crate::model::{MnemonicReport, PassphraseComparison};
use crate::LabResult;

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic_str: &str) -> LabResult<MnemonicReport> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;

    let word_count = m.word_count();
    let checksum_bits = word_count / 3;
    let entropy_bits = (word_count * 32) / 3;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic_str: &str, passphrase: &str) -> LabResult<String> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = m.to_seed(passphrase);
    Ok(hex::encode(seed))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
pub fn compare_passphrases(
    mnemonic_str: &str,
    protected_passphrase: &str,
) -> LabResult<PassphraseComparison> {
    let empty_passphrase_seed_hex = mnemonic_seed_hex(mnemonic_str, "")?;
    let protected_seed_hex = mnemonic_seed_hex(mnemonic_str, protected_passphrase)?;
    let seeds_differ = empty_passphrase_seed_hex != protected_seed_hex;

    Ok(PassphraseComparison {
        empty_passphrase_seed_hex,
        protected_seed_hex,
        seeds_differ,
    })
}

/// Recognize the public BIP39 test mnemonic used in the class labs.
pub fn is_public_test_mnemonic(mnemonic_str: &str) -> bool {
    let words: Vec<&str> = mnemonic_str.split_whitespace().collect();
    if words.len() != 12 {
        return false;
    }
    words[0..11].iter().all(|&w| w == "abandon") && words[11] == "about"
}
