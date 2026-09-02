//! Serialise a Bitcoin transaction from values given on the command line.
//!
//! The pipeline is: text from the command line, checked and turned into bytes
//! by [`cli`], assembled into a [`transaction::Transaction`], serialised by
//! [`transaction::serialize_transaction`] and printed by [`report`].
//!
//! The serialisation itself is the one this program started with. The
//! refactoring moved the transaction data out of the source, not the rules.

pub mod cli;
pub mod error;
pub mod hex;
pub mod report;
pub mod transaction;

pub use cli::{Cli, build_transaction};
pub use error::{Result, TxError};
pub use transaction::{Transaction, TxInput, TxOutput, serialize_transaction};

/// Build, serialise and render a transaction from parsed arguments.
pub fn run(cli: &Cli) -> Result<String> {
    let transaction = build_transaction(cli)?;
    let serialized = serialize_transaction(&transaction);
    Ok(report::render(
        &transaction,
        &serialized,
        cli.verbose,
        cli.bytes,
    ))
}
