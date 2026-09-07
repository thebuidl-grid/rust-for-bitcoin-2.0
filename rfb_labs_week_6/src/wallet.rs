use std::path::Path;
use anyhow::Result;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::Network;
use bdk_wallet::{KeychainKind, PersistedWallet, Wallet};
use bdk_wallet::template::Bip84;
use rusqlite::Connection;

pub type SqliteWallet = PersistedWallet<Connection>;

pub fn open_db(path: &Path) -> Result<Connection> {
    Ok(Connection::open(path)?)
}

pub fn load_or_create_wallet(
    connection: &mut Connection,
    master_private_key: Xpriv,
    network: Network,
) -> Result<SqliteWallet> {
    let external_descriptor = Bip84(master_private_key, KeychainKind::External);
    let internal_descriptor = Bip84(master_private_key, KeychainKind::Internal);

    let wallet_opt = Wallet::load()
        .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
        .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
        .extract_keys()
        .check_network(network)
        .load_wallet(connection)?;

    let wallet = match wallet_opt {
        Some(wallet) => wallet,
        None => Wallet::create(external_descriptor, internal_descriptor)
            .network(network)
            .create_wallet(connection)?
    };

    Ok(wallet)
}

pub fn save(wallet: &mut SqliteWallet, connection: &mut Connection) -> Result<()> {
    wallet.persist(connection)?;
    Ok(())
}