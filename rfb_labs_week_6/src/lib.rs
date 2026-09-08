//! A descriptor-based Bitcoin wallet for regtest / testnet.
//!
//! Library crate so integration tests under `tests/` can drive the same code the
//! `rfbwallet` binary uses.
//!
//! Layering:
//!
//! * [`error`]  — one error type for the whole crate
//! * `config`   — `.env` into a validated [`config::Config`]
//! * `keys`     — mnemonic to extended key to descriptors
//! * `wallet`   — load-or-create the persisted wallet; addresses and balance
//! * `node`     — Bitcoin Core RPC client and the block sync loop
//! * `tx`       — build, sign, broadcast

pub mod cli;
pub mod config;
pub mod error;
pub mod keys;
pub mod node;
pub mod wallet;

pub use error::{Result, WalletError};
