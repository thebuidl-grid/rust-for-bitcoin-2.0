use anyhow::{Context, Result};
use bip39::Mnemonic;
use bdk_wallet::bitcoin::bip32::{DerivationPath, Xpriv};
use bdk_wallet::bitcoin::Network;
use bdk_wallet::bitcoin::secp256k1::rand::RngCore;
use std::str::FromStr;

/// Represents HD Wallet key material and descriptors.
#[derive(Debug, Clone)]
pub struct WalletKeys {
    pub mnemonic: String,
    #[allow(dead_code)]
    pub master_xprv: Xpriv,
    pub external_descriptor: String,
    pub internal_descriptor: String,
    pub descriptor_type: DescriptorType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorType {
    /// BIP84 Native SegWit: wpkh([fingerprint/84'/1'/0']xpub/0/*)
    Wpkh,
    /// BIP86 Taproot: tr([fingerprint/86'/1'/0']xpub/0/*)
    Taproot,
}

impl WalletKeys {
    /// Generates a brand new 12-word BIP39 mnemonic and derives descriptors.
    pub fn generate_new(network: Network, desc_type: DescriptorType) -> Result<Self> {
        let mut entropy = [0u8; 16];
        bdk_wallet::bitcoin::secp256k1::rand::thread_rng().fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy).context("Failed to generate 12-word BIP39 mnemonic")?;
        Self::from_mnemonic(&mnemonic.to_string(), network, desc_type)
    }

    /// Import wallet keys from an existing BIP39 mnemonic phrase.
    pub fn from_mnemonic(phrase: &str, network: Network, desc_type: DescriptorType) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(phrase).context("Invalid BIP39 mnemonic phrase")?;
        let seed = mnemonic.to_seed("");
        
        let master_xprv = Xpriv::new_master(network, &seed)
            .context("Failed to derive master extended private key")?;
        
        let fingerprint = master_xprv.fingerprint(&bdk_wallet::bitcoin::secp256k1::Secp256k1::new());

        let (purpose, coin_type) = match network {
            Network::Bitcoin => (
                match desc_type {
                    DescriptorType::Wpkh => 84,
                    DescriptorType::Taproot => 86,
                },
                0,
            ),
            _ => (
                match desc_type {
                    DescriptorType::Wpkh => 84,
                    DescriptorType::Taproot => 86,
                },
                1,
            ),
        };

        let path_str = format!("m/{purpose}'/{coin_type}'/0'");
        let derivation_path = DerivationPath::from_str(&path_str)?;
        let secp = bdk_wallet::bitcoin::secp256k1::Secp256k1::new();
        let derived_xprv = master_xprv.derive_priv(&secp, &derivation_path)?;

        let desc_prefix = match desc_type {
            DescriptorType::Wpkh => "wpkh",
            DescriptorType::Taproot => "tr",
        };

        // Format descriptors for external (0/*) and internal change (1/*) keychains
        let external_descriptor = format!(
            "{}([{}/{}]{}/0/*)",
            desc_prefix,
            fingerprint,
            path_str.trim_start_matches("m/"),
            derived_xprv
        );

        let internal_descriptor = format!(
            "{}([{}/{}]{}/1/*)",
            desc_prefix,
            fingerprint,
            path_str.trim_start_matches("m/"),
            derived_xprv
        );

        Ok(Self {
            mnemonic: phrase.to_string(),
            master_xprv,
            external_descriptor,
            internal_descriptor,
            descriptor_type: desc_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_derive_wpkh() {
        let keys = WalletKeys::generate_new(Network::Regtest, DescriptorType::Wpkh).unwrap();
        assert_eq!(keys.mnemonic.split_whitespace().count(), 12);
        assert!(keys.external_descriptor.starts_with("wpkh("));
        assert!(keys.internal_descriptor.starts_with("wpkh("));
        assert!(keys.external_descriptor.contains("/0/*"));
        assert!(keys.internal_descriptor.contains("/1/*"));
    }

    #[test]
    fn test_generate_and_derive_taproot() {
        let keys = WalletKeys::generate_new(Network::Regtest, DescriptorType::Taproot).unwrap();
        assert!(keys.external_descriptor.starts_with("tr("));
        assert!(keys.internal_descriptor.starts_with("tr("));
        assert!(keys.external_descriptor.contains("/0/*"));
        assert!(keys.internal_descriptor.contains("/1/*"));
    }
}
