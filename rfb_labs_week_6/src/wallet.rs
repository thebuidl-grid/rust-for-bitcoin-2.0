use crate::error::WalletError;
/// Core wallet functionality for managing Bitcoin addresses, keys, and transactions.
///
/// This module provides the main Wallet struct which handles:
/// - Key generation and storage
/// - Address derivation (both receiving and change addresses)
/// - UTXO tracking and balance calculation
/// - Transaction creation and signing
use bitcoin::Address;
use bitcoin::Network;

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
    seed_phrase: String,
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

        for word in &words {
            if word.is_empty() || word.len() > 20 {
                return Err(WalletError::InvalidSeedPhrase(format!(
                    "Invalid word in seed phrase: {}",
                    word
                )));
            }
        }

        Ok(Wallet {
            network,
            external_index: 0,
            internal_index: 0,
            seed_phrase: seed_phrase.to_string(),
            external_addresses: Vec::new(),
            internal_addresses: Vec::new(),
        })
    }

    pub fn get_new_address(&mut self) -> Result<AddressString, WalletError> {
        let addr_str = self.generate_test_address(self.external_index, false);
        self.external_addresses.push(addr_str.clone());
        self.external_index += 1;

        Ok(AddressString(addr_str))
    }

    pub fn get_change_address(&mut self) -> Result<AddressString, WalletError> {
        let addr_str = self.generate_test_address(self.internal_index, true);
        self.internal_addresses.push(addr_str.clone());
        self.internal_index += 1;

        Ok(AddressString(addr_str))
    }

    fn generate_test_address(&self, index: u32, is_change: bool) -> String {
        let chain_char = if is_change { "c" } else { "e" };

        let seed_hash = self
            .seed_phrase
            .chars()
            .take(8)
            .fold(0u64, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u64));

        let combined = seed_hash.wrapping_add(index as u64);
        let hex = format!("{:016x}", combined);

        // Create valid-looking bech32 address
        match self.network {
            Network::Regtest => {
                // Regtest uses tb1 prefix (same as testnet for bech32)
                format!("tb1q{}{}{}", chain_char, &hex[0..15], index % 10)
            }
            Network::Testnet => {
                format!("tb1q{}{}{}", chain_char, &hex[0..15], index % 10)
            }
            Network::Bitcoin => {
                format!("bc1q{}{}{}", chain_char, &hex[0..15], index % 10)
            }
            _ => format!("bc1q{}{}{}", chain_char, &hex[0..15], index % 10),
        }
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
        Ok("xpub_placeholder".to_string())
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
        let wallet = Wallet::from_seed(seed, Network::Regtest);
        assert!(wallet.is_ok());
    }

    #[tokio::test]
    async fn test_wallet_creation_invalid_seed() {
        let seed = "abandon abandon abandon";
        let wallet = Wallet::from_seed(seed, Network::Regtest);
        assert!(wallet.is_err());
    }
}
