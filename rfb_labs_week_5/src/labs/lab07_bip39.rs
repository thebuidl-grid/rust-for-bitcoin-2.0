//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use bip39::Mnemonic;

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::{LabError, LabResult};

/// The published class test mnemonic. It is known to everyone and must never hold
/// real funds.
pub const PUBLIC_TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// BIP39 spends 11 bits per word and adds one checksum bit per 32 entropy bits, so
/// every three words carry 32 bits of entropy plus one checksum bit.
const ENTROPY_BITS_PER_THREE_WORDS: usize = 32;
const ENTROPY_BITS_PER_CHECKSUM_BIT: usize = 32;

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let parsed = parse_mnemonic(mnemonic)?;
    let word_count = parsed.word_count();
    let entropy_bits = word_count / 3 * ENTROPY_BITS_PER_THREE_WORDS;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits: entropy_bits / ENTROPY_BITS_PER_CHECKSUM_BIT,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    Ok(hex::encode(mnemonic_seed(mnemonic, passphrase)?))
}

/// Derive the raw 64-byte BIP39 seed, which the BIP32 labs use as their root.
///
/// PBKDF2-HMAC-SHA512 over the normalized words, salted with `"mnemonic"` plus the
/// passphrase, for 2048 rounds.
pub fn mnemonic_seed(mnemonic: &str, passphrase: &str) -> LabResult<[u8; 64]> {
    Ok(parse_mnemonic(mnemonic)?.to_seed(passphrase))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
pub fn compare_passphrases(
    mnemonic: &str,
    protected_passphrase: &str,
) -> LabResult<PassphraseComparison> {
    let empty_passphrase_seed_hex = mnemonic_seed_hex(mnemonic, "")?;
    let protected_seed_hex = mnemonic_seed_hex(mnemonic, protected_passphrase)?;

    Ok(PassphraseComparison {
        // The passphrase is stretched into the seed, not stored anywhere, so the same
        // twelve words open a completely separate wallet for every passphrase.
        seeds_differ: empty_passphrase_seed_hex != protected_seed_hex,
        empty_passphrase_seed_hex,
        protected_seed_hex,
    })
}

/// Recognize the public BIP39 test mnemonic used in the class labs.
pub fn is_public_test_mnemonic(mnemonic: &str) -> bool {
    normalize_words(mnemonic) == PUBLIC_TEST_MNEMONIC
}

/// Validate the word list, word count, and checksum.
fn parse_mnemonic(mnemonic: &str) -> LabResult<Mnemonic> {
    Mnemonic::parse(normalize_words(mnemonic))
        .map_err(|error| LabError::InvalidMnemonic(error.to_string()))
}

/// Collapse any run of whitespace into single spaces and lower case the words.
fn normalize_words(mnemonic: &str) -> String {
    mnemonic
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}
