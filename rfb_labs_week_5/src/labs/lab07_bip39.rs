//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use crate::labs::common::parse_mnemonic;
use crate::model::{MnemonicReport, PassphraseComparison};
use crate::LabResult;

/// The public, published BIP39 test vector. It must never hold real funds.
const PUBLIC_TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let parsed = parse_mnemonic(mnemonic)?;
    let word_count = parsed.word_count();

    // BIP39: total bits = words * 11 = ENT + CS, with CS = ENT / 32.
    let groups = word_count / 3;
    Ok(MnemonicReport {
        word_count,
        entropy_bits: groups * 32,
        checksum_bits: groups,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    let parsed = parse_mnemonic(mnemonic)?;
    Ok(hex::encode(parsed.to_seed(passphrase)))
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
    normalized == PUBLIC_TEST_MNEMONIC
}
