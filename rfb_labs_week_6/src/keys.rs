//! Seed material and descriptors.
//!
//! The derivation chain this module implements:
//!
//! ```text
//! entropy (128 or 256 bits)
//!    | BIP39
//! mnemonic  + optional passphrase
//!    | PBKDF2-HMAC-SHA512, 2048 rounds
//! 512-bit seed
//!    | BIP32
//! master xprv
//!    | m/84'/1'/0'   (or m/86'/1'/0' for Taproot)
//! account xprv
//!    | /0/*  and  /1/*
//! external + internal keychains
//! ```
//!
//! The output is a pair of *descriptor strings*. A descriptor fully describes both
//! how to derive every address and how to spend from it, which is why it is the only
//! thing the wallet layer needs.

use bdk_wallet::KeychainKind;
use bdk_wallet::bitcoin::NetworkKind;
use bdk_wallet::bitcoin::bip32::{Fingerprint, Xpriv};
use bdk_wallet::bitcoin::secp256k1::Secp256k1;
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::keys::{DerivableKey, ExtendedKey};
use bdk_wallet::miniscript::Segwitv0;
use bdk_wallet::template::{Bip84, Bip86, DescriptorTemplate};

use crate::config::DescriptorKind;
use crate::error::{Result, WalletError};

/// The descriptor pair that defines a wallet, in both private and public form.
#[derive(Debug, Clone)]
pub struct Descriptors {
    /// External (receive) descriptor **containing the xprv**. Feeds `Wallet`.
    pub external: String,
    /// Internal (change) descriptor containing the xprv.
    pub internal: String,
    /// External descriptor with the xpub only — safe to print, log, or hand to a
    /// watch-only tool such as Bitcoin Core's `importdescriptors`.
    pub external_public: String,
    /// Internal descriptor, xpub only.
    pub internal_public: String,
    /// Master key fingerprint, the wallet's stable identity.
    pub fingerprint: Fingerprint,
    /// Account path these were derived at, e.g. `m/84'/1'/0'`.
    pub account_path: String,
    pub kind: DescriptorKind,
}

/// Generate a fresh mnemonic from the OS random number generator.
pub fn generate_mnemonic(word_count: WordCount) -> Result<Mnemonic> {
    use bdk_wallet::keys::GeneratableKey;

    // The Segwitv0 context is a compile-time marker only; a BIP39 mnemonic is not
    // script-type specific. It has no effect on the words produced.
    let generated =
        <Mnemonic as GeneratableKey<Segwitv0>>::generate((word_count, Language::English)).map_err(
            |e| {
                WalletError::MnemonicGeneration(
                    e.map(|err| err.to_string())
                        .unwrap_or_else(|| "entropy source failed".to_string()),
                )
            },
        )?;

    Ok(generated.into_key())
}

/// Parse a user-supplied mnemonic, tolerating sloppy whitespace and casing.
///
/// The BIP39 English wordlist is pure ASCII, so lowercasing and collapsing runs of
/// whitespace is all the normalisation required.
pub fn parse_mnemonic(phrase: &str) -> Result<Mnemonic> {
    let normalised = phrase
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    Ok(Mnemonic::parse_in_normalized(Language::English, &normalised)?)
}

/// Mnemonic (+ optional passphrase) to BIP32 master extended private key.
///
/// `NetworkKind` decides only the serialisation prefix (`xprv` vs `tprv`); the
/// underlying key material is identical either way.
pub fn master_xprv(
    mnemonic: &Mnemonic,
    passphrase: Option<&str>,
    network_kind: NetworkKind,
) -> Result<Xpriv> {
    let key: (Mnemonic, Option<String>) = (mnemonic.clone(), passphrase.map(str::to_string));

    let extended: ExtendedKey<Segwitv0> = key.into_extended_key()?;

    // `None` here means the key was public-only. Deriving from a mnemonic always
    // yields a private key, so this is defensive rather than expected.
    extended
        .into_xprv(network_kind)
        .ok_or(WalletError::NoPrivateKey)
}

/// Build the external/internal descriptor pair for the chosen script type.
pub fn derive_descriptors(
    mnemonic: &Mnemonic,
    passphrase: Option<&str>,
    network_kind: NetworkKind,
    kind: DescriptorKind,
) -> Result<Descriptors> {
    let xprv = master_xprv(mnemonic, passphrase, network_kind)?;

    // Build each keychain separately. The template appends the full BIP44-style
    // path plus the keychain branch: /0/* for external, /1/* for internal.
    // `Xpriv` is `Copy`, so the same master key feeds both.
    let (external, ext_keys, _) = build(xprv, kind, KeychainKind::External, network_kind)?;
    let (internal, int_keys, _) = build(xprv, kind, KeychainKind::Internal, network_kind)?;

    let secp = Secp256k1::new();
    let coin_type = match network_kind {
        NetworkKind::Main => 0,
        NetworkKind::Test => 1,
    };

    Ok(Descriptors {
        // `to_string_with_secret` re-inserts the xprv from the key map. Plain
        // `to_string` prints the xpub form, because the parsed descriptor holds
        // only public keys — the secrets live alongside it in the key map.
        external: external.to_string_with_secret(&ext_keys),
        internal: internal.to_string_with_secret(&int_keys),
        external_public: external.to_string(),
        internal_public: internal.to_string(),
        fingerprint: xprv.fingerprint(&secp),
        account_path: format!("m/{}'/{coin_type}'/0'", kind.purpose()),
        kind,
    })
}

/// Expand one keychain through the appropriate BIP template.
fn build(
    xprv: Xpriv,
    kind: DescriptorKind,
    keychain: KeychainKind,
    network_kind: NetworkKind,
) -> Result<bdk_wallet::descriptor::template::DescriptorTemplateOut> {
    let out = match kind {
        DescriptorKind::Wpkh => Bip84(xprv, keychain).build(network_kind)?,
        DescriptorKind::Tr => Bip86(xprv, keychain).build(network_kind)?,
    };
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical BIP39 all-zeros test vector. Published in the spec, so it is
    /// safe to commit — it is not, and must never be, anyone's real seed.
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn vector() -> Mnemonic {
        parse_mnemonic(TEST_MNEMONIC).unwrap()
    }

    #[test]
    fn generated_mnemonics_have_the_requested_length() {
        assert_eq!(
            generate_mnemonic(WordCount::Words12).unwrap().words().count(),
            12
        );
        assert_eq!(
            generate_mnemonic(WordCount::Words24).unwrap().words().count(),
            24
        );
    }

    #[test]
    fn generated_mnemonics_are_unique() {
        let a = generate_mnemonic(WordCount::Words12).unwrap();
        let b = generate_mnemonic(WordCount::Words12).unwrap();
        assert_ne!(a.to_string(), b.to_string(), "entropy source is not random");
    }

    #[test]
    fn parsing_tolerates_untidy_input() {
        let messy = "  ABANDON abandon\tabandon abandon abandon abandon \n abandon abandon abandon abandon abandon ABOUT ";
        assert_eq!(parse_mnemonic(messy).unwrap(), vector());
    }

    #[test]
    fn rejects_a_bad_checksum() {
        let bad = TEST_MNEMONIC.replace("about", "abandon");
        assert!(matches!(parse_mnemonic(&bad), Err(WalletError::Mnemonic(_))));
    }

    #[test]
    fn descriptors_are_deterministic() {
        let d1 =
            derive_descriptors(&vector(), None, NetworkKind::Test, DescriptorKind::Wpkh).unwrap();
        let d2 =
            derive_descriptors(&vector(), None, NetworkKind::Test, DescriptorKind::Wpkh).unwrap();
        assert_eq!(d1.external, d2.external);
        assert_eq!(d1.fingerprint, d2.fingerprint);
        assert_eq!(d1.account_path, "m/84'/1'/0'");
    }

    #[test]
    fn keychains_are_separate() {
        let d =
            derive_descriptors(&vector(), None, NetworkKind::Test, DescriptorKind::Wpkh).unwrap();
        assert_ne!(d.external, d.internal, "external and internal must differ");
        // BDK expands the keychain branch into the path: /0/* vs /1/*.
        assert!(d.external_public.contains("/0/*"), "{}", d.external_public);
        assert!(d.internal_public.contains("/1/*"), "{}", d.internal_public);
    }

    #[test]
    fn public_descriptors_carry_no_secret() {
        let d =
            derive_descriptors(&vector(), None, NetworkKind::Test, DescriptorKind::Wpkh).unwrap();
        assert!(d.external_public.contains("tpub"), "{}", d.external_public);
        assert!(!d.external_public.contains("tprv"), "leaked a private key");
        assert!(d.external.contains("tprv"), "signing descriptor needs the xprv");
    }

    #[test]
    fn script_kinds_produce_different_descriptors() {
        let w =
            derive_descriptors(&vector(), None, NetworkKind::Test, DescriptorKind::Wpkh).unwrap();
        let t = derive_descriptors(&vector(), None, NetworkKind::Test, DescriptorKind::Tr).unwrap();
        assert!(
            w.external_public.starts_with("wpkh("),
            "{}",
            w.external_public
        );
        assert!(t.external_public.starts_with("tr("), "{}", t.external_public);
        assert_eq!(t.account_path, "m/86'/1'/0'");
        // Same seed, so the same master key identity, different derivation path.
        assert_eq!(w.fingerprint, t.fingerprint);
    }

    #[test]
    fn a_passphrase_changes_everything() {
        let plain =
            derive_descriptors(&vector(), None, NetworkKind::Test, DescriptorKind::Wpkh).unwrap();
        let salted = derive_descriptors(
            &vector(),
            Some("correct horse"),
            NetworkKind::Test,
            DescriptorKind::Wpkh,
        )
        .unwrap();
        assert_ne!(plain.fingerprint, salted.fingerprint);
    }
}
