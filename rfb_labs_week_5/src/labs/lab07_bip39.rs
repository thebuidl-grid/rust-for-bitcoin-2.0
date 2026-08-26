//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use bip39::Mnemonic;

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::{LabError, LabResult};

/// The published, public BIP39 test mnemonic used throughout these labs. It is known to
/// everyone and must never be used to protect real funds.
pub const PUBLIC_TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn parse_mnemonic(mnemonic: &str) -> LabResult<Mnemonic> {
    Mnemonic::parse(mnemonic).map_err(|error| LabError::InvalidMnemonic(error.to_string()))
}

/// Validate an English mnemonic and report its entropy/checksum structure.
///
/// `Mnemonic::parse` validates the checksum as part of parsing: the checksum is error
/// detection (catches typos and mis-transcriptions), not encryption — anyone who reads
/// the words has the entropy, checksum included.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let parsed = parse_mnemonic(mnemonic)?;

    let word_count = parsed.word_count();
    let total_bits = word_count * 11;
    let entropy_bits = parsed.to_entropy().len() * 8;
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
    Ok(hex::encode(parsed.to_seed(passphrase)))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
///
/// A forgotten passphrase cannot be recovered from the mnemonic alone: the passphrase is
/// mixed into the PBKDF2 derivation itself (it is not stored anywhere in the words or
/// their checksum), so a different — or missing — passphrase silently selects a
/// completely different wallet rather than failing loudly.
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
    let normalized = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized == PUBLIC_TEST_MNEMONIC
}
