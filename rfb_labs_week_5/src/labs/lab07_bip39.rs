//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use bip39::{Language, Mnemonic};

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::{LabError, LabResult};

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let normalized = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");

    let parsed = Mnemonic::parse_in(Language::English, &normalized)
        .map_err(|error| LabError::InvalidMnemonic(format!("invalid BIP39 mnemonic: {error}")))?;

    let word_count = parsed.word_count();

    // BIP39 entropy and checksum relationship:
    // ENT (bits) | CS (bits) | total (bits) | word count
    //     96     |     3     |      99      |     12
    //    128     |     4     |     132      |     15
    //    160     |     5     |     165      |     18
    //    192     |     6     |     198      |     21
    //    224     |     7     |     231      |     24
    //    256     |     8     |     264      |     (not standard)
    //
    // Each word encodes 11 bits, so total_bits = word_count * 11
    // checksum_bits = total_bits / 33
    // entropy_bits = total_bits - checksum_bits

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
    let normalized = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");

    let parsed = Mnemonic::parse_in(Language::English, &normalized)
        .map_err(|error| LabError::InvalidMnemonic(format!("invalid BIP39 mnemonic: {error}")))?;

    let seed = parsed.to_seed(passphrase);

    Ok(hex::encode(seed))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
pub fn compare_passphrases(
    mnemonic: &str,
    protected_passphrase: &str,
) -> LabResult<PassphraseComparison> {
    let empty_seed_hex = mnemonic_seed_hex(mnemonic, "")?;
    let protected_seed_hex = mnemonic_seed_hex(mnemonic, protected_passphrase)?;

    let seeds_differ = empty_seed_hex != protected_seed_hex;

    Ok(PassphraseComparison {
        empty_passphrase_seed_hex: empty_seed_hex,
        protected_seed_hex,
        seeds_differ,
    })
}

/// Recognize the public BIP39 test mnemonic used in the class labs.
pub fn is_public_test_mnemonic(mnemonic: &str) -> bool {
    let normalized = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized == "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
}
