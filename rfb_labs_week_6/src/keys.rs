use std::str::FromStr;

use bdk_wallet::bitcoin::Network;
use bdk_wallet::bitcoin::NetworkKind;
use bdk_wallet::bitcoin::bip32::DerivationPath;
use bdk_wallet::bitcoin::secp256k1::{All, Secp256k1};
use bdk_wallet::descriptor;
use bdk_wallet::descriptor::IntoWalletDescriptor;
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::keys::{GeneratableKey, GeneratedKey};
// The `descriptor!` macro's `wpkh` arm expands to an unqualified `miniscript::`
// path, so the crate has to be in scope at the call site.
use bdk_wallet::miniscript;
use bdk_wallet::miniscript::Tap;

use crate::error::WalletError;

// === Descriptor kind

/// The two single-signature descriptor shapes this wallet supports. Both keep an
/// external (receive) and internal (change) keychain; they differ only in the
/// output type and BIP purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorKind {
    /// `wpkh(...)`, native SegWit v0, BIP84 (`m/84h/...`).
    Wpkh,
    /// `tr(...)`, Taproot key-spend, BIP86 (`m/86h/...`).
    Tr,
}

impl DescriptorKind {
    fn purpose(self) -> u32 {
        return match self {
            DescriptorKind::Wpkh => 84,
            DescriptorKind::Tr => 86,
        };
    }

    pub fn as_str(self) -> &'static str {
        return match self {
            DescriptorKind::Wpkh => "wpkh",
            DescriptorKind::Tr => "tr",
        };
    }
}

impl FromStr for DescriptorKind {
    type Err = WalletError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return match s.trim().to_ascii_lowercase().as_str() {
            "wpkh" | "bip84" | "segwit" => Ok(DescriptorKind::Wpkh),
            "tr" | "taproot" | "bip86" => Ok(DescriptorKind::Tr),
            other => Err(WalletError::Config(format!(
                "unknown DESCRIPTOR_KIND '{other}', expected 'wpkh' or 'tr'"
            ))),
        };
    }
}

// === Derivation

/// A pair of string descriptors, each carrying its extended private key so BDK
/// can both watch and sign. These strings are never written to disk by this
/// crate; they are re-derived from the mnemonic on every run.
pub struct WalletDescriptors {
    pub external: String,
    pub internal: String,
}

/// Generate a fresh 12 word English mnemonic.
pub fn generate_mnemonic() -> Result<String, WalletError> {
    let mnemonic: GeneratedKey<_, Tap> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|_| WalletError::Mnemonic("mnemonic generation failed".to_string()))?;
    return Ok((*mnemonic).to_string());
}

/// Derive the external and internal descriptors for `kind` from a BIP39 mnemonic.
///
/// Test networks (testnet, signet, regtest) all use coin type `1h`; only mainnet
/// uses `0h`, and mainnet is rejected earlier in config loading.
pub fn derive_descriptors(
    mnemonic_words: &str,
    passphrase: Option<&str>,
    kind: DescriptorKind,
    network: Network,
) -> Result<WalletDescriptors, WalletError> {
    let secp = Secp256k1::new();

    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic_words.trim())
        .map_err(|e| WalletError::Mnemonic(e.to_string()))?;

    let net_kind = match network {
        Network::Bitcoin => NetworkKind::Main,
        _ => NetworkKind::Test,
    };
    let coin_type = match network {
        Network::Bitcoin => 0u32,
        _ => 1u32,
    };
    let purpose = kind.purpose();

    let external_path = path(purpose, coin_type, 0)?;
    let internal_path = path(purpose, coin_type, 1)?;

    let passphrase = passphrase.map(str::to_string);
    let external = descriptor_string(
        kind,
        (mnemonic.clone(), passphrase.clone()),
        external_path,
        &secp,
        net_kind,
    )?;
    let internal = descriptor_string(kind, (mnemonic, passphrase), internal_path, &secp, net_kind)?;

    return Ok(WalletDescriptors { external, internal });
}

fn path(purpose: u32, coin_type: u32, change: u32) -> Result<DerivationPath, WalletError> {
    let raw = format!("m/{purpose}h/{coin_type}h/0h/{change}");
    return DerivationPath::from_str(&raw).map_err(|e| WalletError::Descriptor(e.to_string()));
}

fn descriptor_string(
    kind: DescriptorKind,
    key: (Mnemonic, Option<String>),
    derivation: DerivationPath,
    secp: &Secp256k1<All>,
    net: NetworkKind,
) -> Result<String, WalletError> {
    let built = match kind {
        DescriptorKind::Wpkh => descriptor!(wpkh((key, derivation))),
        DescriptorKind::Tr => descriptor!(tr((key, derivation))),
    }
    .map_err(|e| WalletError::Descriptor(e.to_string()))?;

    let (descriptor, key_map) = built
        .into_wallet_descriptor(secp, net)
        .map_err(|e| WalletError::Descriptor(e.to_string()))?;

    return Ok(descriptor.to_string_with_secret(&key_map));
}
