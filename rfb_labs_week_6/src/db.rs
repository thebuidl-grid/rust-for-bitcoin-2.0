/// Database module for persisting wallet state.
///
/// This module handles all SQLite operations to store:
/// - Wallet metadata (network, xpub)
/// - Derived addresses and their derivation paths
/// - UTXOs (unspent transaction outputs)
/// - Transaction history
///
/// By persisting this data, the wallet can be closed and reopened
/// without needing to re-sync everything from the blockchain.
use crate::error::{Result, WalletError};
use rusqlite::{params, Connection};
use std::path::Path;

/// Database manager for wallet operations.
///
/// Handles SQLite operations for storing wallet state.
pub struct Database {
    /// SQLite database connection
    conn: Connection,
}

impl Database {
    /// Creates a new database connection, initializing schema if needed.
    ///
    /// # How it works:
    /// 1. Connects to SQLite file at the provided path
    /// 2. Creates tables if they don't exist (idempotent)
    /// 3. Returns a Database instance ready for queries
    ///
    /// # Tables created:
    /// - `wallet_info`: Metadata about the wallet (network, xpub)
    /// - `addresses`: All derived addresses and their types (internal/external)
    /// - `utxos`: Unspent transaction outputs
    /// - `transactions`: Transaction history
    pub async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(&db_path)
            .map_err(|e| WalletError::DatabaseError(format!("Failed to open database: {}", e)))?;

        let db = Database { conn };
        db.init_schema()?;

        Ok(db)
    }

    /// Initializes database schema (creates tables if they don't exist).
    ///
    /// This is idempotent - running it multiple times is safe.
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS wallet_info (
                    id INTEGER PRIMARY KEY,
                    network TEXT NOT NULL,
                    xpub TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS addresses (
                    id INTEGER PRIMARY KEY,
                    address TEXT UNIQUE NOT NULL,
                    derivation_path TEXT NOT NULL,
                    chain_type TEXT NOT NULL,
                    address_index INTEGER NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS utxos (
                    id INTEGER PRIMARY KEY,
                    txid TEXT NOT NULL,
                    output_index INTEGER NOT NULL,
                    amount_satoshis INTEGER NOT NULL,
                    address TEXT NOT NULL,
                    is_spent BOOLEAN DEFAULT FALSE,
                    confirmed BOOLEAN DEFAULT FALSE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(txid, output_index)
                );

                CREATE TABLE IF NOT EXISTS transactions (
                    id INTEGER PRIMARY KEY,
                    txid TEXT UNIQUE NOT NULL,
                    tx_hex TEXT NOT NULL,
                    amount_satoshis INTEGER NOT NULL,
                    fee_satoshis INTEGER NOT NULL,
                    tx_type TEXT NOT NULL,
                    confirmed BOOLEAN DEFAULT FALSE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                "#,
            )
            .map_err(|e| {
                WalletError::DatabaseError(format!("Failed to initialize schema: {}", e))
            })?;

        Ok(())
    }

    /// Stores wallet metadata (network and xpub).
    pub async fn save_wallet_info(&self, network: &str, xpub: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO wallet_info (id, network, xpub) VALUES (1, ?1, ?2)",
                params![network, xpub],
            )
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Stores a derived address in the database.
    pub async fn save_address(
        &self,
        address: &str,
        derivation_path: &str,
        chain_type: &str,
        index: u32,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO addresses (address, derivation_path, chain_type, address_index) VALUES (?1, ?2, ?3, ?4)",
                params![address, derivation_path, chain_type, index as i32],
            )
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Gets all addresses of a specific type (internal or external).
    pub async fn get_addresses(&self, chain_type: &str) -> Result<Vec<(String, u32)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT address, address_index FROM addresses WHERE chain_type = ?1 ORDER BY address_index")
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        let addresses = stmt
            .query_map(params![chain_type], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)? as u32))
            })
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(addresses)
    }

    /// Saves a UTXO to the database.
    pub async fn save_utxo(
        &self,
        txid: &str,
        output_index: u32,
        amount_satoshis: u64,
        address: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO utxos (txid, output_index, amount_satoshis, address, confirmed) VALUES (?1, ?2, ?3, ?4, TRUE)",
                params![txid, output_index as i32, amount_satoshis as i64, address],
            )
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Marks a UTXO as spent.
    pub async fn mark_utxo_spent(&self, txid: &str, output_index: u32) -> Result<()> {
        self.conn
            .execute(
                "UPDATE utxos SET is_spent = TRUE WHERE txid = ?1 AND output_index = ?2",
                params![txid, output_index as i32],
            )
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Gets all unspent UTXOs.
    pub async fn get_unspent_utxos(&self) -> Result<Vec<UtxoRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT txid, output_index, amount_satoshis, address, confirmed FROM utxos WHERE is_spent = FALSE ORDER BY amount_satoshis DESC")
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        let utxos = stmt
            .query_map([], |row| {
                Ok(UtxoRecord {
                    txid: row.get(0)?,
                    output_index: row.get(1)?,
                    amount_satoshis: row.get(2)?,
                    address: row.get(3)?,
                    confirmed: row.get(4)?,
                })
            })
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(utxos)
    }

    /// Calculates the total confirmed balance.
    pub async fn get_balance(&self) -> Result<u64> {
        let balance: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount_satoshis), 0) FROM utxos WHERE is_spent = FALSE AND confirmed = TRUE",
                [],
                |row| row.get(0),
            )
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(balance as u64)
    }

    /// Saves a transaction to the database.
    pub async fn save_transaction(
        &self,
        txid: &str,
        tx_hex: &str,
        amount: u64,
        fee: u64,
        tx_type: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO transactions (txid, tx_hex, amount_satoshis, fee_satoshis, tx_type, confirmed) VALUES (?1, ?2, ?3, ?4, ?5, FALSE)",
                params![txid, tx_hex, amount as i64, fee as i64, tx_type],
            )
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Gets transaction history.
    pub async fn get_transactions(&self, limit: i64) -> Result<Vec<TransactionRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT txid, amount_satoshis, fee_satoshis, tx_type, confirmed, created_at FROM transactions ORDER BY created_at DESC LIMIT ?1")
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        let transactions = stmt
            .query_map(params![limit], |row| {
                Ok(TransactionRecord {
                    txid: row.get(0)?,
                    amount_satoshis: row.get(1)?,
                    fee_satoshis: row.get(2)?,
                    tx_type: row.get(3)?,
                    confirmed: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(transactions)
    }
}

#[derive(Debug, Clone)]
pub struct UtxoRecord {
    pub txid: String,
    pub output_index: i32,
    pub amount_satoshis: i64,
    pub address: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub txid: String,
    pub amount_satoshis: i64,
    pub fee_satoshis: i64,
    pub tx_type: String,
    pub confirmed: bool,
    pub created_at: Option<String>,
}
