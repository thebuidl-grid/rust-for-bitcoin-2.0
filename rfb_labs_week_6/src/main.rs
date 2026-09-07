use anyhow::Result;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::Network;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, Wallet};
use dotenvy::dotenv;
use rand::RngCore;
use std::env;

const WALLET_DB: &str = "wallet.sqlite";

fn generate_key() -> Result<()> {
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);

    let xprv = Xpriv::new_master(Network::Regtest, &seed)?;

    println!("Generated disposable regtest xprv:");
    println!("{}", xprv);
    println!();
    println!("Add this to .env as:");
    println!("WALLET_XPRV={}", xprv);

    Ok(())
}

fn main() -> Result<()> {
    dotenv().ok();

    if env::args().nth(1).as_deref() == Some("generate-key") {
        return generate_key();
    }

    let xprv = env::var("WALLET_XPRV")
        .expect("WALLET_XPRV must be set in .env");

    let external_descriptor =
        format!("wpkh({}/84'/1'/0'/0/*)", xprv);

    let internal_descriptor =
        format!("wpkh({}/84'/1'/0'/1/*)", xprv);

    let mut conn = Connection::open(WALLET_DB)?;

    let mut wallet = match Wallet::load()
        .descriptor(
            KeychainKind::External,
            Some(external_descriptor.clone()),
        )
        .descriptor(
            KeychainKind::Internal,
            Some(internal_descriptor.clone()),
        )
        .extract_keys()
        .check_network(Network::Regtest)
        .load_wallet(&mut conn)?
    {
        Some(wallet) => {
            println!("Loaded existing wallet.");
            wallet
        }

        None => {
            println!("Creating new wallet...");

            Wallet::create(
                external_descriptor,
                internal_descriptor,
            )
            .network(Network::Regtest)
            .create_wallet(&mut conn)?
        }
    };

    let receive = wallet.reveal_next_address(KeychainKind::External);
    let change = wallet.reveal_next_address(KeychainKind::Internal);

    wallet.persist(&mut conn)?;

    println!();
    println!("=== Wallet ===");
    println!("Network: regtest");
    println!("Receive address: {}", receive.address);
    println!("Change address:  {}", change.address);

    Ok(())
}