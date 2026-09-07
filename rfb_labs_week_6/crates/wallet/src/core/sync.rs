use crate::{
    core::WalletService,
    error::{WalletError, WalletResult},
    types::{WalletBalance, WalletUtxo},
};

impl WalletService {
    pub fn sync(&mut self) -> WalletResult<()> {
        Err(WalletError::NotImplemented("wallet synchronization"))
    }

    pub fn balance(&self) -> WalletResult<WalletBalance> {
        Err(WalletError::NotImplemented("balance"))
    }

    pub fn list_utxos(&self) -> WalletResult<Vec<WalletUtxo>> {
        Err(WalletError::NotImplemented("UTXO listing"))
    }
}
