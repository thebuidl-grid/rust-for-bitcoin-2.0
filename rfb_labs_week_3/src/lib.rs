//! Week 3 — Understanding Bitcoin Data.
//!
//! The real work lives in the two member crates:
//!
//! - [`decodetrx`] — decodes a raw transaction hex into JSON, with a Clap CLI.
//! - [`trxparse`] — parses a raw transaction into a JSON object.
//!
//! This crate exists so the integration tests in `tests/` have a package to
//! hang off. See `tests/decodetrx.rs` and `tests/trxparse.rs`.

pub use decodetrx;
pub use trxparse;
