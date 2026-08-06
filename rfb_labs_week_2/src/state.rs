//! Part 10 (optional) — transaction lifecycle as a typestate machine.
//!
//! A transaction moves through a fixed sequence of states:
//!
//! ```text
//! Created ──validate()──> Validated ──sign()──> Signed ──broadcast()──> Broadcast
//!    │                                                                      │
//!    └──────────────── Rejected <────────── reject() ───────────────────────┤
//!                                                                           │
//!                                              Confirmed <───confirm()──────┘
//! ```
//!
//! The state is carried in the *type*, not in a field, so an invalid transition is
//! a compile error rather than a runtime check. There is no `broadcast()` method on
//! a `Created` transaction to call in the first place, so the mistake cannot reach
//! production, cannot be reached by a test, and needs no `if` to guard it.
//!
//! This is the same lesson as Part 7 applied to state instead of memory: let the
//! compiler make the wrong thing unrepresentable.

use std::fmt;

use crate::{error::TransactionError, transaction::Transaction};

/// Built but not yet checked.
#[derive(Debug, PartialEq, Eq)]
pub struct Created;

/// Passed [`Transaction::validate`].
#[derive(Debug, PartialEq, Eq)]
pub struct Validated;

/// Signed and ready to send. The signature is a placeholder; this model does not
/// perform real cryptography.
#[derive(Debug, PartialEq, Eq)]
pub struct Signed {
    pub signature: String,
}

/// Sent to the network and sitting in the mempool.
#[derive(Debug, PartialEq, Eq)]
pub struct Broadcast;

/// Included in a block.
#[derive(Debug, PartialEq, Eq)]
pub struct Confirmed {
    pub block_height: u32,
}

/// Refused, either by validation or by the network.
#[derive(Debug, PartialEq, Eq)]
pub struct Rejected {
    pub reason: RejectionReason,
}

/// Why a transaction was rejected.
#[derive(Debug, PartialEq, Eq)]
pub enum RejectionReason {
    /// Failed local validation before it was ever sent.
    ValidationFailed(TransactionError),
    /// Accepted locally but refused by the network.
    RefusedByNetwork(String),
}

impl fmt::Display for RejectionReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationFailed(error) => write!(formatter, "validation failed: {error}"),
            Self::RefusedByNetwork(reason) => write!(formatter, "refused by network: {reason}"),
        }
    }
}

/// A transaction together with its lifecycle state.
///
/// `S` is the current state. Each transition consumes `self` and returns a value
/// of a different type, so the previous state is gone — you cannot broadcast the
/// same transaction twice, because the value you would need no longer exists.
#[derive(Debug, PartialEq, Eq)]
pub struct Lifecycle<S> {
    transaction: Transaction,
    state: S,
}

impl<S> Lifecycle<S> {
    /// Read-only access in any state.
    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    /// The state marker itself, for states that carry data.
    pub fn state(&self) -> &S {
        &self.state
    }
}

impl Lifecycle<Created> {
    pub fn new(transaction: Transaction) -> Self {
        Self {
            transaction,
            state: Created,
        }
    }

    /// The only route out of `Created`.
    ///
    /// Returns `Rejected` rather than an error type, because a rejected
    /// transaction is still a transaction — it has a place in the lifecycle and
    /// its contents remain inspectable.
    ///
    /// ```
    /// use rfb_labs_week_2::{Lifecycle, Transaction};
    ///
    /// let lifecycle = Lifecycle::new(Transaction::new(2, 0));
    /// // An empty transaction has no inputs, so validation rejects it.
    /// assert!(lifecycle.validate().is_err());
    /// ```
    #[allow(clippy::result_large_err)]
    pub fn validate(self) -> Result<Lifecycle<Validated>, Lifecycle<Rejected>> {
        match self.transaction.validate() {
            Ok(()) => Ok(Lifecycle {
                transaction: self.transaction,
                state: Validated,
            }),
            Err(error) => Err(Lifecycle {
                transaction: self.transaction,
                state: Rejected {
                    reason: RejectionReason::ValidationFailed(error),
                },
            }),
        }
    }
}

impl Lifecycle<Validated> {
    /// Signing is only reachable from `Validated`, so an unvalidated transaction
    /// can never be signed.
    pub fn sign(self, signature: impl Into<String>) -> Lifecycle<Signed> {
        Lifecycle {
            transaction: self.transaction,
            state: Signed {
                signature: signature.into(),
            },
        }
    }
}

impl Lifecycle<Signed> {
    pub fn broadcast(self) -> Lifecycle<Broadcast> {
        Lifecycle {
            transaction: self.transaction,
            state: Broadcast,
        }
    }
}

impl Lifecycle<Broadcast> {
    /// A broadcast transaction either makes it into a block...
    pub fn confirm(self, block_height: u32) -> Lifecycle<Confirmed> {
        Lifecycle {
            transaction: self.transaction,
            state: Confirmed { block_height },
        }
    }

    /// ...or is dropped by the network.
    pub fn reject(self, reason: impl Into<String>) -> Lifecycle<Rejected> {
        Lifecycle {
            transaction: self.transaction,
            state: Rejected {
                reason: RejectionReason::RefusedByNetwork(reason.into()),
            },
        }
    }
}

impl Lifecycle<Confirmed> {
    pub fn block_height(&self) -> u32 {
        self.state.block_height
    }
}

impl Lifecycle<Rejected> {
    pub fn reason(&self) -> &RejectionReason {
        &self.state.reason
    }
}

/// Names each state for display. Implemented per state so the name comes from the
/// type rather than from a field that could disagree with it.
pub trait StateName {
    const NAME: &'static str;
}

impl StateName for Created {
    const NAME: &'static str = "Created";
}
impl StateName for Validated {
    const NAME: &'static str = "Validated";
}
impl StateName for Signed {
    const NAME: &'static str = "Signed";
}
impl StateName for Broadcast {
    const NAME: &'static str = "Broadcast";
}
impl StateName for Confirmed {
    const NAME: &'static str = "Confirmed";
}
impl StateName for Rejected {
    const NAME: &'static str = "Rejected";
}

impl<S: StateName> fmt::Display for Lifecycle<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{}] {}", S::NAME, self.transaction)
    }
}

/// Invalid transitions do not compile.
///
/// Broadcasting straight from `Created` skips validation and signing, so no such
/// method exists:
///
/// ```compile_fail
/// use rfb_labs_week_2::{Lifecycle, Transaction};
///
/// let lifecycle = Lifecycle::new(Transaction::new(2, 0));
/// lifecycle.broadcast();
/// ```
///
/// Neither does signing before validating:
///
/// ```compile_fail
/// use rfb_labs_week_2::{Lifecycle, Transaction};
///
/// let lifecycle = Lifecycle::new(Transaction::new(2, 0));
/// lifecycle.sign("signature");
/// ```
///
/// And a transaction cannot be broadcast twice, because the first call consumed it:
///
/// ```compile_fail
/// use rfb_labs_week_2::{Lifecycle, Transaction};
///
/// # fn example(signed: Lifecycle<rfb_labs_week_2::Signed>) {
/// let broadcast = signed.broadcast();
/// let again = signed.broadcast();
/// # }
/// ```
#[derive(Debug)]
pub struct InvalidTransitionsDoNotCompile;
