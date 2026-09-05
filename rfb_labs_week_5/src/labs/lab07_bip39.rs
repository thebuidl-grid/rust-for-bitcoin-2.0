//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use bip39::Mnemonic;

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::{LabError, LabResult};

const PUBLIC_TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn parse_mnemonic(mnemonic: &str) -> LabResult<Mnemonic> {
    Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))
}

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let mnemonic = parse_mnemonic(mnemonic)?;
    let word_count = mnemonic.word_count();
    let entropy_bits = mnemonic.to_entropy().len() * 8;
    // BIP39: CS = ENT / 32 bits, and MS (word count) = (ENT + CS) / 11, so CS = word_count / 3.
    let checksum_bits = word_count / 3;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    let mnemonic = parse_mnemonic(mnemonic)?;
    Ok(hex::encode(mnemonic.to_seed(passphrase)))
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
    let normalized: Vec<&str> = mnemonic.split_whitespace().collect();
    normalized.join(" ") == PUBLIC_TEST_MNEMONIC
}
