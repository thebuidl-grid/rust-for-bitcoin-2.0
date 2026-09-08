//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use bip39::Mnemonic;

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::LabResult;
use crate::LabError;

/// The public class test mnemonic (11x abandon + about).
const PUBLIC_TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Validate an English mnemonic and report its entropy/checksum structure.
/// For BIP39: total_bits = word_count * 11, checksum_bits = total_bits / 33.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let m = Mnemonic::parse_normalized(mnemonic)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;

    let word_count = m.word_count();
    let total_bits = word_count * 11;
    let checksum_bits = total_bits / 33;
    let entropy_bits = total_bits - checksum_bits;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    let m = Mnemonic::parse_normalized(mnemonic)
        .map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;

    let seed = m.to_seed(passphrase);
    Ok(hex::encode(seed))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
pub fn compare_passphrases(
    mnemonic: &str,
    protected_passphrase: &str,
) -> LabResult<PassphraseComparison> {
    let empty_seed = mnemonic_seed_hex(mnemonic, "")?;
    let protected_seed = mnemonic_seed_hex(mnemonic, protected_passphrase)?;
    let seeds_differ = empty_seed != protected_seed;

    Ok(PassphraseComparison {
        empty_passphrase_seed_hex: empty_seed,
        protected_seed_hex: protected_seed,
        seeds_differ,
    })
}

/// Recognize the public BIP39 test mnemonic used in the class labs.
/// Accepts the canonical 12-word "abandon x11 + about" after normalizing whitespace.
pub fn is_public_test_mnemonic(mnemonic: &str) -> bool {
    let normalized: String = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized == PUBLIC_TEST_MNEMONIC
}
