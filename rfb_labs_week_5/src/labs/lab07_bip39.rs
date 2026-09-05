//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::LabResult;
use bip39::{Language, Mnemonic};

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let normalized = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");

    let mnemonic = Mnemonic::parse_in(Language::English, &normalized)
        .map_err(|error| crate::LabError::InvalidMnemonic(error.to_string()))?;

    let word_count = mnemonic.word_count();
    let entropy_bits = mnemonic.to_entropy().len() * 8;
    let checksum_bits = entropy_bits / 32;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    let normalized = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");

    let mnemonic = Mnemonic::parse_in(Language::English, &normalized)
        .map_err(|error| crate::LabError::InvalidMnemonic(error.to_string()))?;

    let seed = mnemonic.to_seed(passphrase);

    Ok(hex::encode(seed))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
pub fn compare_passphrases(
    mnemonic: &str,
    protected_passphrase: &str,
) -> LabResult<PassphraseComparison> {
    let empty_passphrase_seed_hex = mnemonic_seed_hex(mnemonic, "")?;
    let protected_seed_hex = mnemonic_seed_hex(mnemonic, protected_passphrase)?;

    Ok(PassphraseComparison {
        seeds_differ: empty_passphrase_seed_hex != protected_seed_hex,
        empty_passphrase_seed_hex,
        protected_seed_hex,
    })
}

/// Recognize the public BIP39 test mnemonic used in the class labs.
pub fn is_public_test_mnemonic(mnemonic: &str) -> bool {
    let normalized = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");

    normalized
        == "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
}
