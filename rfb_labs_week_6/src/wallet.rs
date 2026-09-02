use anyhow::{Context, Result};
use bdk_wallet::bitcoin::{Address, Amount, Network, Transaction};
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, LocalOutput, PersistedWallet, Wallet};
use std::path::Path;
use std::str::FromStr;

pub struct WalletManager {
    pub wallet: PersistedWallet<Connection>,
    pub db: Connection,
    pub network: Network,
}

impl WalletManager {
    /// Opens an SQLite database and loads an existing wallet or creates a new one from descriptors.
    pub fn open_or_create<P: AsRef<Path>>(
        db_path: P,
        external_desc: &str,
        internal_desc: Option<&str>,
        network: Network,
    ) -> Result<Self> {
        let mut db = Connection::open(db_path.as_ref())
            .with_context(|| format!("Failed to open SQLite database at {:?}", db_path.as_ref()))?;

        let wallet_opt = Wallet::load()
            .descriptor(KeychainKind::External, Some(external_desc.to_string()))
            .descriptor(KeychainKind::Internal, internal_desc.map(|s| s.to_string()))
            .extract_keys()
            .check_network(network)
            .load_wallet(&mut db)?;

        let wallet = match wallet_opt {
            Some(w) => w,
            None => match internal_desc {
                Some(int_desc) => Wallet::create(external_desc.to_string(), int_desc.to_string())
                    .network(network)
                    .create_wallet(&mut db)?,
                None => Wallet::create_single(external_desc.to_string())
                    .network(network)
                    .create_wallet(&mut db)?,
            },
        };

        Ok(Self {
            wallet,
            db,
            network,
        })
    }

    /// Reveal next unused receiving address (external keychain).
    pub fn get_new_address(&mut self) -> Result<Address> {
        let info = self.wallet.reveal_next_address(KeychainKind::External);
        self.wallet.persist(&mut self.db)?;
        Ok(info.address)
    }

    /// Reveal next unused change address (internal keychain).
    pub fn get_change_address(&mut self) -> Result<Address> {
        let info = self.wallet.reveal_next_address(KeychainKind::Internal);
        self.wallet.persist(&mut self.db)?;
        Ok(info.address)
    }

    /// Get current balance breakdown.
    pub fn balance(&self) -> bdk_wallet::Balance {
        self.wallet.balance()
    }

    /// List unspent transaction outputs (UTXOs).
    pub fn list_utxos(&self) -> Vec<LocalOutput> {
        self.wallet.list_unspent().collect()
    }

    /// Construct, sign, and extract a transaction to send funds to a destination address.
    pub fn create_and_sign_tx(&mut self, recipient: &str, amount_sats: u64) -> Result<Transaction> {
        let to_address = Address::from_str(recipient)
            .context("Invalid recipient address")?
            .require_network(self.network)
            .context("Recipient address network mismatch")?;

        let mut tx_builder = self.wallet.build_tx();
        tx_builder.add_recipient(to_address.script_pubkey(), Amount::from_sat(amount_sats));

        let mut psbt = tx_builder.finish()?;
        
        let finalized = self.wallet.sign(&mut psbt, bdk_wallet::SignOptions::default())?;
        if !finalized {
            anyhow::bail!("Failed to finalize PSBT transaction signatures");
        }

        self.wallet.persist(&mut self.db)?;

        let tx = psbt.extract_tx()?;
        Ok(tx)
    }

    /// Persist current wallet state to SQLite database.
    pub fn save(&mut self) -> Result<()> {
        self.wallet.persist(&mut self.db)?;
        Ok(())
    }
}
