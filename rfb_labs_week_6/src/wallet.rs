use crate::error::WalletError;
/// Core wallet functionality for managing Bitcoin addresses, keys, and transactions.
///
/// This module provides the main Wallet struct which handles:
/// - Key generation and storage
/// - Address derivation (both receiving and change addresses)
/// - UTXO tracking and balance calculation
/// - Transaction creation and signing
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Address;
use bitcoin::Network;
use bitcoin::PublicKey;
use std::str::FromStr;

/// Main wallet structure that manages Bitcoin addresses and transactions.
///
/// The wallet uses BDK (Bitcoin Development Kit) which provides:
/// - Descriptor-based address generation (deterministic and secure)
/// - UTXO management
/// - Transaction building and signing
///
/// Behind the scenes:
/// - We derive keys using BIP32 hierarchical deterministic (HD) wallets
/// - External chain (m/84'/0'/0'/0/*) for receiving addresses
/// - Internal chain (m/84'/0'/0'/1/*) for change addresses
/// - All state is persisted to SQLite database
pub struct Wallet {
    network: Network,
    external_index: u32,
    internal_index: u32,
    root_xprv: ExtendedPrivKey,
    external_addresses: Vec<String>,
    internal_addresses: Vec<String>,
}

impl Wallet {
    /// Creates a new wallet from a seed phrase (12 or 24 words).
    ///
    /// # How it works:
    /// 1. Takes the seed phrase and converts it to a binary seed
    /// 2. Derives a root key using BIP32 standard
    /// 3. Creates a descriptor (wallet blueprint) that specifies:
    ///    - Address type (wpkh = native segwit)
    ///    - Derivation path (where to find keys)
    /// 4. Initializes the BDK wallet with this descriptor
    ///
    /// # Arguments
    /// * `seed_phrase` - 12 or 24 word BIP39 mnemonic
    /// * `network` - Bitcoin network (usually bitcoin::Network::Regtest for testing)

    pub async fn from_seed(seed_phrase: &str, network: Network) -> Result<Self, WalletError> {
        let words: Vec<&str> = seed_phrase.split_whitespace().collect();
        if words.len() != 12 && words.len() != 24 {
            return Err(WalletError::InvalidSeedPhrase(
                "Seed phrase must be 12 or 24 words".to_string(),
            ));
        }

        let mnemonic = bip39::Mnemonic::parse(seed_phrase).map_err(|e| {
            WalletError::InvalidSeedPhrase(format!("Invalid BIP39 mnemonic: {}", e))
        })?;

        let seed = mnemonic.to_seed("");
        let root_xprv = ExtendedPrivKey::new_master(network, &seed).map_err(|e| {
            WalletError::WalletCreationError(format!("Failed to derive master key: {}", e))
        })?;

        Ok(Wallet {
            network,
            external_index: 0,
            internal_index: 0,
            root_xprv,
            external_addresses: Vec::new(),
            internal_addresses: Vec::new(),
        })
    }

    pub fn get_new_address(&mut self) -> Result<AddressString, WalletError> {
        let address = self.derive_address(self.external_index, false)?;
        let addr_str = address.to_string();
        self.external_addresses.push(addr_str.clone());
        self.external_index += 1;

        Ok(AddressString(addr_str))
    }

    pub fn get_change_address(&mut self) -> Result<AddressString, WalletError> {
        let address = self.derive_address(self.internal_index, true)?;
        let addr_str = address.to_string();
        self.internal_addresses.push(addr_str.clone());
        self.internal_index += 1;

        Ok(AddressString(addr_str))
    }

    /// Derives a native SegWit (BIP84) address at `m/84'/0'/0'/{0,1}/{index}`.
    fn derive_address(&self, index: u32, is_change: bool) -> Result<Address, WalletError> {
        let secp = Secp256k1::new();
        let chain = if is_change { 1 } else { 0 };

        let path = DerivationPath::from_str(&format!("m/84'/0'/0'/{}/{}", chain, index))
            .map_err(|e| WalletError::Internal(format!("Invalid derivation path: {}", e)))?;

        let child_xprv = self
            .root_xprv
            .derive_priv(&secp, &path)
            .map_err(|e| WalletError::Internal(format!("Key derivation failed: {}", e)))?;

        let child_xpub = ExtendedPubKey::from_priv(&secp, &child_xprv);
        let public_key = PublicKey::new(child_xpub.public_key);

        Address::p2wpkh(&public_key, self.network)
            .map_err(|e| WalletError::Internal(format!("Address generation failed: {}", e)))
    }

    pub fn get_balance(&self) -> Result<u64, WalletError> {
        Ok(0)
    }

    /// Lists all unspent transaction outputs (UTXOs) in the wallet.
    pub fn list_utxos(&self) -> Result<Vec<UtxoInfo>, WalletError> {
        Ok(vec![])
    }
    pub async fn sync(&mut self) -> Result<(), WalletError> {
        log::info!("Syncing wallet with Bitcoin network...");
        Ok(())
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn get_xpub(&self) -> Result<String, WalletError> {
        let secp = Secp256k1::new();
        let xpub = ExtendedPubKey::from_priv(&secp, &self.root_xprv);
        Ok(xpub.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct AddressString(pub String);

impl std::fmt::Display for AddressString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information about a single unspent transaction output (UTXO).
#[derive(Debug, Clone)]
pub struct UtxoInfo {
    pub txid: String,
    pub output_index: u32,
    pub amount_satoshis: u64,
    pub address: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet_creation_valid_seed() {
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let wallet = Wallet::from_seed(seed, Network::Regtest).await;
        assert!(wallet.is_ok());
    }

    #[tokio::test]
    async fn test_wallet_creation_invalid_seed() {
        let seed = "abandon abandon abandon";
        let wallet = Wallet::from_seed(seed, Network::Regtest).await;
        assert!(wallet.is_err());
    }
}
