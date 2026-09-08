/// Custom error types for the Bitcoin wallet application.
///
/// This module defines all the different errors that can occur during wallet operations.
/// Using specific error types makes it easier to handle different failure scenarios appropriately.
use thiserror::Error;

/// Custom error type for all wallet operations.
///
/// By using custom errors, callers can distinguish between:
/// - Invalid input (bad seed phrase, malformed address, etc.)
/// - Database errors (can't read/write to SQLite)
/// - Network errors (can't connect to Bitcoin node)
/// - Cryptocurrency-specific errors (invalid transaction, signing failed, etc.)
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Invalid seed phrase: {0}")]
    InvalidSeedPhrase(String),

    #[error("Wallet creation failed: {0}")]
    WalletCreationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Signing error: {0}")]
    SigningError(String),

    #[error("Insufficient funds: available {available}, requested {required}")]
    InsufficientFunds { available: u64, required: u64 },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for WalletError: Instead of using Result<T, WalletError> everywhere, this Result<T> helps instead.
pub type Result<T> = std::result::Result<T, WalletError>;
