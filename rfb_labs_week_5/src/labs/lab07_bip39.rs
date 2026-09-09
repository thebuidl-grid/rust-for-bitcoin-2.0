//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use bip39::Mnemonic;

use crate::error::LabError;
use crate::model::{MnemonicReport, PassphraseComparison};
use crate::LabResult;

const PUBLIC_TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn parse_mnemonic(mnemonic: &str) -> LabResult<Mnemonic> {
    mnemonic
        .parse::<Mnemonic>()
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))
}

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let parsed = parse_mnemonic(mnemonic)?;
    let word_count = parsed.word_count();
    let entropy_bits = parsed.to_entropy().len() * 8;
    let total_bits = word_count * 11;
    let checksum_bits = total_bits - entropy_bits;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    let parsed = parse_mnemonic(mnemonic)?;
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
    let normalized: Vec<&str> = mnemonic.split_whitespace().collect();
    let expected: Vec<&str> = PUBLIC_TEST_MNEMONIC.split_whitespace().collect();
    normalized == expected
}
