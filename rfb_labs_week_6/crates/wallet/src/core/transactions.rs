use bitcoin::{Amount, FeeRate};

use crate::{
    core::WalletService,
    error::{WalletError, WalletResult},
    types::TransactionSummary,
};

impl WalletService {
    pub fn send(
        &mut self,
        _destination: &str,
        _amount: Amount,
        _fee_rate: FeeRate,
    ) -> WalletResult<TransactionSummary> {
        Err(WalletError::NotImplemented("transaction sending"))
    }
}
