//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::{LabError, LabResult};
use bip39::Mnemonic;

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    // todo!("Lab 07: validate BIP39 words and calculate ENT/CS lengths")
    let m = Mnemonic::parse(mnemonic).map_err(|_| LabError::InvalidMnemonic(mnemonic.into()))?;
    let word_count = m.word_count();
    let entropy_bits = (word_count * 11) * 32 / 33;
    let checksum_bits = word_count * 11 - entropy_bits;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    // todo!("Lab 07: apply the BIP39 mnemonic-to-seed function")
    let m = Mnemonic::parse(mnemonic).map_err(|_| LabError::InvalidMnemonic(mnemonic.into()))?;
    let seed = m.to_seed(passphrase);
    Ok(hex::encode(seed))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
pub fn compare_passphrases(
    mnemonic: &str,
    protected_passphrase: &str,
) -> LabResult<PassphraseComparison> {
    // todo!("Lab 07: compare the empty-passphrase and protected seeds")
    let empty = mnemonic_seed_hex(mnemonic, "")?;
    let protected_passphrase = mnemonic_seed_hex(mnemonic, protected_passphrase)?;

    Ok(PassphraseComparison {
        empty_passphrase_seed_hex: empty.clone(),
        protected_seed_hex: protected_passphrase.clone(),
        seeds_differ: empty != protected_passphrase,
    })
}

/// Recognize the public BIP39 test mnemonic used in the class labs.
pub fn is_public_test_mnemonic(mnemonic: &str) -> bool {
    // todo!("Lab 07: accept only abandon x11 + about after normalizing whitespace")
    let normalized = mnemonic
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase();

    normalized == "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
}
