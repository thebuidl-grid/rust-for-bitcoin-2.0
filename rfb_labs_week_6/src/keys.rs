use std::str::FromStr;

use bdk_wallet::bitcoin::bip32::DerivationPath;
use bdk_wallet::bitcoin::secp256k1::Secp256k1;
use bdk_wallet::bitcoin::NetworkKind;
use bdk_wallet::descriptor;
use bdk_wallet::descriptor::IntoWalletDescriptor;
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::keys::{GeneratableKey, GeneratedKey};
use bdk_wallet::miniscript;
use bdk_wallet::miniscript::Segwitv0;

use crate::error::WalletError;

/// BIP84 (native segwit) account-0 paths, one keychain each.
const EXTERNAL_PATH: &str = "m/84h/1h/0h/0";
const INTERNAL_PATH: &str = "m/84h/1h/0h/1";

pub struct Descriptors {
    /// Private-key-embedded descriptors (tprv-based) — used to build a
    /// signing-capable wallet. Never print or log these.
    pub external: String,
    pub internal: String,
    /// Public-only equivalents (xpub-based), safe to print/log.
    pub external_public: String,
    pub internal_public: String,
}

/// Parses `existing` as a mnemonic if present, otherwise generates a fresh
/// 12-word one and prints a one-time reminder to save it to `.env`.
pub fn load_or_generate_mnemonic(existing: Option<&str>) -> Result<Mnemonic, WalletError> {
    if let Some(phrase) = existing {
        return Mnemonic::parse_in(Language::English, phrase)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()));
    }

    let generated: GeneratedKey<Mnemonic, Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|_| WalletError::MnemonicGeneration)?;
    let mnemonic: Mnemonic = (*generated).clone();

    eprintln!(
        "No MNEMONIC configured — generated a fresh one for this run:\n  {mnemonic}\n\
         Save it to .env as MNEMONIC=\"...\" to reuse this wallet on the next run.\n\
         This is a test-only key: never reuse it on mainnet, never commit it."
    );

    Ok(mnemonic)
}

/// Derives the external (receive) and internal (change) wpkh descriptors
/// for `mnemonic` on `network`, following BIP84.
pub fn descriptors_from_mnemonic(
    mnemonic: &Mnemonic,
    network: NetworkKind,
) -> Result<Descriptors, WalletError> {
    let secp = Secp256k1::new();
    // EXTERNAL_PATH/INTERNAL_PATH are compile-time constants, not user
    // input — `expect` is safe here because there's no runtime value
    // that could make this parse fail.
    let external_path = DerivationPath::from_str(EXTERNAL_PATH).expect("EXTERNAL_PATH is valid");
    let internal_path = DerivationPath::from_str(INTERNAL_PATH).expect("INTERNAL_PATH is valid");
    let mnemonic_with_passphrase = (mnemonic.clone(), None::<String>);

    let (external, ext_keymap) =
        descriptor!(wpkh((mnemonic_with_passphrase.clone(), external_path)))
            .map_err(|e| WalletError::DescriptorBuild(e.to_string()))?
            .into_wallet_descriptor(&secp, network)
            .map_err(|e| WalletError::DescriptorBuild(e.to_string()))?;
    let (internal, int_keymap) = descriptor!(wpkh((mnemonic_with_passphrase, internal_path)))
        .map_err(|e| WalletError::DescriptorBuild(e.to_string()))?
        .into_wallet_descriptor(&secp, network)
        .map_err(|e| WalletError::DescriptorBuild(e.to_string()))?;

    // `Descriptor::to_string()` only serializes the public half — the
    // private key material lives in the separate KeyMap. A wallet built
    // from public-only descriptor strings is watch-only and can't sign,
    // so we must embed the private keys with `to_string_with_secret`.
    // Keep the plain `to_string()` (xpub-only) form around too, for
    // anything that wants to print/log a descriptor safely.
    Ok(Descriptors {
        external: external.to_string_with_secret(&ext_keymap),
        internal: internal.to_string_with_secret(&int_keymap),
        external_public: external.to_string(),
        internal_public: internal.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_mnemonic_round_trips_through_parsing() {
        let mnemonic = load_or_generate_mnemonic(None).unwrap();
        let phrase = mnemonic.to_string();
        let reparsed = load_or_generate_mnemonic(Some(&phrase)).unwrap();
        assert_eq!(mnemonic, reparsed);
    }

    #[test]
    fn invalid_mnemonic_is_rejected() {
        let err = load_or_generate_mnemonic(Some("not a real mnemonic")).unwrap_err();
        assert!(matches!(err, WalletError::InvalidMnemonic(_)));
    }

    #[test]
    fn external_and_internal_descriptors_differ_and_parse() {
        let mnemonic = load_or_generate_mnemonic(None).unwrap();
        let descriptors = descriptors_from_mnemonic(&mnemonic, NetworkKind::Test).unwrap();

        assert_ne!(descriptors.external, descriptors.internal);

        let secp = Secp256k1::new();
        descriptors
            .external
            .clone()
            .into_wallet_descriptor(&secp, NetworkKind::Test)
            .expect("external descriptor should parse");
        descriptors
            .internal
            .clone()
            .into_wallet_descriptor(&secp, NetworkKind::Test)
            .expect("internal descriptor should parse");
    }

    #[test]
    fn same_mnemonic_derives_deterministic_descriptors() {
        let mnemonic = load_or_generate_mnemonic(None).unwrap();
        let first = descriptors_from_mnemonic(&mnemonic, NetworkKind::Test).unwrap();
        let second = descriptors_from_mnemonic(&mnemonic, NetworkKind::Test).unwrap();
        assert_eq!(first.external, second.external);
        assert_eq!(first.internal, second.internal);
    }
}