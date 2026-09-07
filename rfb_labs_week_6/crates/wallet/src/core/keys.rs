use crate::{
    core::WalletService,
    error::{WalletError, WalletResult},
    types::{DerivedAddress, Keychain},
};

impl WalletService {
    pub fn initialize() -> WalletResult<Self> {
        Err(WalletError::NotImplemented("wallet initialization"))
    }

    pub fn next_address(&mut self, _keychain: Keychain) -> WalletResult<DerivedAddress> {
        Err(WalletError::NotImplemented("address derivation"))
    }
}
