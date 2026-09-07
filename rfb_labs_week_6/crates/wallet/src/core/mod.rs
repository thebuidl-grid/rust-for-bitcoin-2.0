mod keys;
mod sync;
mod transactions;

/// Coordinates descriptor wallet operations without knowing about CLI parsing.
///
/// BDK wallet creation/loading and persistence will be wired into this service in
/// the next implementation stage.
pub struct WalletService;
