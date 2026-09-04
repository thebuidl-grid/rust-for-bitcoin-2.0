use bdk_wallet::bitcoin::{Amount, OutPoint};
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{AddressInfo, Balance, KeychainKind, PersistedWallet, Wallet as BdkWallet};

use crate::config::Config;
use crate::error::WalletError;
use crate::keys::{self, WalletDescriptors};

/// A BDK wallet bound to its SQLite store. The descriptors are re-derived from
/// the configured mnemonic every time, so the only thing that has to survive a
/// restart is the chain data (checkpoints, tx graph, keychain indexes), and that
/// lives in `db`.
pub struct Wallet {
    inner: PersistedWallet<Connection>,
    db: Connection,
}

/// One tracked unspent output, flattened for display.
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: Amount,
    pub keychain: KeychainKind,
    pub confirmed: bool,
}

impl Wallet {
    /// Create a brand new wallet database. Fails if one already exists so we
    /// never clobber existing state.
    pub fn create(config: &Config) -> Result<Self, WalletError> {
        if config.db_path.exists() {
            return Err(WalletError::WalletAlreadyExists(config.db_path.clone()));
        }

        let descriptors = derive(config)?;
        let mut db = Connection::open(&config.db_path)?;

        let inner = BdkWallet::create(descriptors.external, descriptors.internal)
            .network(config.network)
            .create_wallet(&mut db)
            .map_err(|e| WalletError::Load(e.to_string()))?;

        return Ok(Wallet { inner, db });
    }

    /// Open an existing wallet database. The provided descriptors are checked
    /// against what was persisted, so a changed mnemonic is caught here.
    pub fn open(config: &Config) -> Result<Self, WalletError> {
        if !config.db_path.exists() {
            return Err(WalletError::WalletNotInitialized(config.db_path.clone()));
        }

        let descriptors = derive(config)?;
        let mut db = Connection::open(&config.db_path)?;

        let loaded = BdkWallet::load()
            .descriptor(KeychainKind::External, Some(descriptors.external))
            .descriptor(KeychainKind::Internal, Some(descriptors.internal))
            .extract_keys()
            .check_network(config.network)
            .load_wallet(&mut db)
            .map_err(|e| WalletError::Load(e.to_string()))?;

        let inner = loaded.ok_or_else(|| {
            WalletError::Load(
                "wallet database exists but holds no wallet; delete it and run `init`".to_string(),
            )
        })?;

        return Ok(Wallet { inner, db });
    }

    /// Reveal the next unused external (receiving) address and persist the new
    /// index so it is not handed out again after a restart.
    pub fn new_receive_address(&mut self) -> Result<AddressInfo, WalletError> {
        let info = self.inner.reveal_next_address(KeychainKind::External);
        self.persist()?;
        return Ok(info);
    }

    pub fn balance(&self) -> Balance {
        return self.inner.balance();
    }

    pub fn utxos(&self) -> Vec<Utxo> {
        return self
            .inner
            .list_unspent()
            .map(|out| Utxo {
                outpoint: out.outpoint,
                value: out.txout.value,
                keychain: out.keychain,
                confirmed: out.chain_position.is_confirmed(),
            })
            .collect();
    }

    /// The public descriptors, safe to print or share.
    pub fn public_descriptors(&self) -> (String, String) {
        return (
            self.inner
                .public_descriptor(KeychainKind::External)
                .to_string(),
            self.inner
                .public_descriptor(KeychainKind::Internal)
                .to_string(),
        );
    }

    // Access for the sync and tx modules.
    pub(crate) fn inner_mut(&mut self) -> &mut PersistedWallet<Connection> {
        return &mut self.inner;
    }

    pub(crate) fn inner(&self) -> &PersistedWallet<Connection> {
        return &self.inner;
    }

    /// Flush staged changes to SQLite.
    pub(crate) fn persist(&mut self) -> Result<(), WalletError> {
        self.inner.persist(&mut self.db)?;
        return Ok(());
    }
}

fn derive(config: &Config) -> Result<WalletDescriptors, WalletError> {
    return keys::derive_descriptors(
        &config.mnemonic,
        config.passphrase.as_deref(),
        config.descriptor_kind,
        config.network,
    );
}
