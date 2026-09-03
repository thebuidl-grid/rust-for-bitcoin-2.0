use anyhow::{Context, Result, anyhow};
use bdk_wallet::bitcoin::bip32::DerivationPath;
use bdk_wallet::bitcoin::secp256k1::Secp256k1;
use bdk_wallet::bitcoin::{Network, NetworkKind};
use bdk_wallet::descriptor;
use bdk_wallet::descriptor::IntoWalletDescriptor;
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::keys::{GeneratableKey, GeneratedKey};
use bdk_wallet::miniscript;
use bdk_wallet::miniscript::Segwitv0;
use std::str::FromStr;

use crate::config::ScriptType;

/// A wallet's external (receive) and internal (change) descriptors, each in both
/// private (holds the signing key, never persisted to the wallet DB) and public form.
pub struct DescriptorPair {
    pub external_private: String,
    pub internal_private: String,
    pub external_public: String,
    pub internal_public: String,
}

/// Load a mnemonic from `.env`/CLI, or generate and persist a fresh one to `.env`.
///
/// Returns the mnemonic and whether it was freshly generated.
pub fn load_or_generate_mnemonic(existing: Option<String>) -> Result<(Mnemonic, bool)> {
    if let Some(phrase) = existing {
        let mnemonic = Mnemonic::parse(&phrase).context("invalid MNEMONIC")?;
        return Ok((mnemonic, false));
    }

    let generated: GeneratedKey<_, Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|_| anyhow!("failed to generate mnemonic"))?;
    let mnemonic: Mnemonic = (*generated).clone();
    persist_mnemonic_to_env(&mnemonic.to_string())?;
    Ok((mnemonic, true))
}

fn persist_mnemonic_to_env(phrase: &str) -> Result<()> {
    use std::io::Write;

    let path = ".env";
    let mut existing = std::fs::read_to_string(path).unwrap_or_default();
    if existing.contains("MNEMONIC=") {
        return Ok(());
    }
    if !existing.is_empty() && !existing.ends_with('\n') {
        existing.push('\n');
    }
    existing.push_str(&format!("MNEMONIC=\"{phrase}\"\n"));
    let mut file = std::fs::File::create(path)?;
    file.write_all(existing.as_bytes())?;
    Ok(())
}

/// Derive external/internal wallet descriptors for `mnemonic` at `m/purpose'/coin_type'/account'/{0,1}`,
/// where `purpose` and the descriptor script type follow `script_type`, and `coin_type` follows
/// `network` (0' for mainnet, 1' for every test network, per BIP44).
pub fn derive_descriptors(
    mnemonic: &Mnemonic,
    network: Network,
    account: u32,
    script_type: ScriptType,
) -> Result<DescriptorPair> {
    let secp = Secp256k1::new();
    let network_kind = NetworkKind::from(network);
    let coin_type = if network == Network::Bitcoin { 0 } else { 1 };
    let mnemonic_with_passphrase = (mnemonic.clone(), None);

    match script_type {
        ScriptType::Wpkh => {
            let external_path =
                DerivationPath::from_str(&format!("m/84'/{coin_type}'/{account}'/0"))?;
            let internal_path =
                DerivationPath::from_str(&format!("m/84'/{coin_type}'/{account}'/1"))?;

            let (external_descriptor, ext_keymap) =
                descriptor!(wpkh((mnemonic_with_passphrase.clone(), external_path)))?
                    .into_wallet_descriptor(&secp, network_kind)?;
            let (internal_descriptor, int_keymap) =
                descriptor!(wpkh((mnemonic_with_passphrase, internal_path)))?
                    .into_wallet_descriptor(&secp, network_kind)?;

            Ok(DescriptorPair {
                external_public: external_descriptor.to_string(),
                internal_public: internal_descriptor.to_string(),
                external_private: external_descriptor.to_string_with_secret(&ext_keymap),
                internal_private: internal_descriptor.to_string_with_secret(&int_keymap),
            })
        }
        ScriptType::Tr => {
            let external_path =
                DerivationPath::from_str(&format!("m/86'/{coin_type}'/{account}'/0"))?;
            let internal_path =
                DerivationPath::from_str(&format!("m/86'/{coin_type}'/{account}'/1"))?;

            let (external_descriptor, ext_keymap) =
                descriptor!(tr((mnemonic_with_passphrase.clone(), external_path)))?
                    .into_wallet_descriptor(&secp, network_kind)?;
            let (internal_descriptor, int_keymap) =
                descriptor!(tr((mnemonic_with_passphrase, internal_path)))?
                    .into_wallet_descriptor(&secp, network_kind)?;

            Ok(DescriptorPair {
                external_public: external_descriptor.to_string(),
                internal_public: internal_descriptor.to_string(),
                external_private: external_descriptor.to_string_with_secret(&ext_keymap),
                internal_private: internal_descriptor.to_string_with_secret(&int_keymap),
            })
        }
    }
}
