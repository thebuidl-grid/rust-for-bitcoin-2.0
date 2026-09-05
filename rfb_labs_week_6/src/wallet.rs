use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{AddressInfo, Balance, KeychainKind, LocalOutput, PersistedWallet, Wallet};
use bitcoin::Network;

use crate::error::WalletError;

pub type WalletDb = Connection;

/// Loads a previously-persisted wallet from `db`, or creates a fresh one
/// if `db` has no wallet data yet.
pub fn open_or_create_wallet(
    external_descriptor: String,
    internal_descriptor: String,
    network: Network,
    db: &mut WalletDb,
) -> Result<PersistedWallet<WalletDb>, WalletError> {
    let loaded = Wallet::load()
        .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
        .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
        .extract_keys()
        .check_network(network)
        .load_wallet(db)
        .map_err(|e| WalletError::Persistence(e.to_string()))?;

    match loaded {
        Some(wallet) => Ok(wallet),
        None => Wallet::create(external_descriptor, internal_descriptor)
            .network(network)
            .create_wallet(db)
            .map_err(|e| WalletError::Persistence(e.to_string())),
    }
}

pub fn new_receive_address(wallet: &mut PersistedWallet<WalletDb>) -> AddressInfo {
    wallet.reveal_next_address(KeychainKind::External)
}

pub fn new_change_address(wallet: &mut PersistedWallet<WalletDb>) -> AddressInfo {
    wallet.reveal_next_address(KeychainKind::Internal)
}

pub fn get_balance(wallet: &PersistedWallet<WalletDb>) -> Balance {
    wallet.balance()
}

pub fn list_utxos(wallet: &PersistedWallet<WalletDb>) -> Vec<LocalOutput> {
    wallet.list_unspent().collect()
}