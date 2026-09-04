// House style for this crate: every function uses an explicit `return`, so the
// `needless_return` lint is turned off here rather than fought line by line.
#![allow(clippy::needless_return)]

// === Modules
pub mod config;
pub mod error;
pub mod keys;
pub mod node;
pub mod raw_demo;
pub mod sync;
pub mod tx;
pub mod wallet;

pub use error::WalletError;

pub type Result<T> = core::result::Result<T, WalletError>;
