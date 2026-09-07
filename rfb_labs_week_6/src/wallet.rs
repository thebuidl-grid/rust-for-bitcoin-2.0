//! Creating, opening and persisting the BDK wallet.
//!
//! The invariant this module enforces: the SQLite file holds public data only.
//! Private keys are rebuilt from the environment mnemonic on every open, which
//! means an attacker with the database file gets the wallet's transaction
//! history and nothing more. It also means the wallet opens perfectly well
//! without a mnemonic, just without the ability to sign.

use std::path::Path;

use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, PersistedWallet, Wallet};
use bitcoin::bip32::Fingerprint;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::keys::{self, DescriptorKind};
use crate::store;

/// An open wallet together with the connection it persists through.
pub struct WalletHandle {
    pub wallet: PersistedWallet<Connection>,
    pub conn: Connection,
    pub kind: DescriptorKind,
    /// Whether private keys were loaded, i.e. whether this wallet can sign.
    pub signing: bool,
}

impl WalletHandle {
    /// Flush staged changes to SQLite. Returns whether anything was written.
    pub fn persist(&mut self) -> Result<bool> {
        Ok(self.wallet.persist(&mut self.conn)?)
    }

    pub fn require_signing(&self) -> Result<()> {
        if self.signing {
            Ok(())
        } else {
            Err(Error::MissingMnemonic)
        }
    }
}

/// Create a brand new wallet database.
pub fn create(cfg: &Config, kind: DescriptorKind, birthday: u32) -> Result<WalletHandle> {
    if cfg.db_path.exists() {
        return Err(Error::WalletAlreadyExists(cfg.db_path.clone()));
    }
    cfg.ensure_db_dir()?;

    let mnemonic = cfg.require_mnemonic()?;
    let descriptors = keys::descriptors(
        mnemonic,
        cfg.passphrase(),
        cfg.network,
        cfg.coin_type(),
        kind,
    )?;

    let mut conn = store::open(&cfg.db_path)?;
    let params = Wallet::create(descriptors.external.clone(), descriptors.internal.clone())
        .network(cfg.network)
        .lookahead(50);

    let wallet = PersistedWallet::create(&mut conn, params)
        .map_err(|e| Error::wallet("creating the wallet database", e))?;

    store::set(&conn, store::KEY_DESCRIPTOR_KIND, kind.as_str())?;
    store::set(&conn, store::KEY_BIRTHDAY, &birthday.to_string())?;
    store::set(
        &conn,
        store::KEY_FINGERPRINT,
        &descriptors.master_fingerprint.to_string(),
    )?;
    store::set(&conn, store::KEY_CREATED_AT, &unix_now().to_string())?;

    Ok(WalletHandle {
        wallet,
        conn,
        kind,
        signing: true,
    })
}

/// Open an existing wallet database.
///
/// If a mnemonic is available the private descriptors are rebuilt and checked
/// against what was persisted, so a mismatched seed is caught at open time
/// rather than at signing time. Without a mnemonic the wallet opens read-only.
pub fn load(cfg: &Config) -> Result<WalletHandle> {
    if !cfg.db_path.exists() {
        return Err(Error::WalletNotInitialised(cfg.db_path.clone()));
    }

    let mut conn = store::open(&cfg.db_path)?;
    let kind = store::descriptor_kind(&conn)?;

    let mut params = Wallet::load().check_network(cfg.network);
    let signing = match cfg.mnemonic() {
        Some(mnemonic) => {
            let descriptors = keys::descriptors(
                mnemonic,
                cfg.passphrase(),
                cfg.network,
                cfg.coin_type(),
                kind,
            )?;
            params = params
                .descriptor(KeychainKind::External, Some(descriptors.external))
                .descriptor(KeychainKind::Internal, Some(descriptors.internal))
                .extract_keys();
            true
        }
        None => false,
    };

    let wallet = PersistedWallet::load(&mut conn, params)
        .map_err(|e| Error::wallet("opening the wallet database", e))?
        .ok_or_else(|| Error::WalletNotInitialised(cfg.db_path.clone()))?;

    Ok(WalletHandle {
        wallet,
        conn,
        kind,
        signing,
    })
}

/// The master fingerprint recorded at creation time, if present.
pub fn recorded_fingerprint(conn: &Connection) -> Result<Option<Fingerprint>> {
    let Some(raw) = store::get(conn, store::KEY_FINGERPRINT)? else {
        return Ok(None);
    };
    raw.parse::<Fingerprint>()
        .map(Some)
        .map_err(|e| Error::wallet("reading the stored master fingerprint", e))
}

pub fn db_size(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
