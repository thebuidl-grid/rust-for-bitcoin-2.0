use bitcoincore_rpc::{Auth, Client, RpcApi};
use bdk_bitcoind_rpc::{Emitter, NO_EXPECTED_MEMPOOL_TXS};
use anyhow::Result;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, Wallet};
use bdk_wallet::bitcoin::{Address, Amount, Network};
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

    let rpc_url = env::var("RPC_URL")?;
let rpc_user = env::var("RPC_USER")?;
let rpc_password = env::var("RPC_PASSWORD")?;

let rpc = Client::new(
    &rpc_url,
    Auth::UserPass(rpc_user, rpc_password),
)?;

let blockchain_info = rpc.get_blockchain_info()?;

println!(
    "Connected to Bitcoin Core: {} at height {}",
    blockchain_info.chain, blockchain_info.blocks
);

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

    let wallet_tip = wallet.latest_checkpoint();

let mut emitter = Emitter::new(
    &rpc,
    wallet_tip.clone(),
    wallet_tip.height(),
    NO_EXPECTED_MEMPOOL_TXS,
);

println!("Syncing wallet...");

while let Some(block) = emitter.next_block()? {
    wallet.apply_block_connected_to(
        &block.block,
        block.block_height(),
        block.connected_to(),
    )?;
}

let mempool = emitter.mempool()?.update;
wallet.apply_unconfirmed_txs(mempool);

println!("Wallet synced.");

    let receive = wallet.reveal_next_address(KeychainKind::External);
    let change = wallet.reveal_next_address(KeychainKind::Internal);

    wallet.persist(&mut conn)?;

    let balance = wallet.balance();

println!();
println!("=== Balance ===");
println!("Total: {}", balance.total());

println!();
println!("=== UTXOs ===");

for utxo in wallet.list_unspent() {
    println!(
        "{}:{} — {} BTC",
        utxo.outpoint.txid,
        utxo.outpoint.vout,
        utxo.txout.value
    );
}

    println!();
    println!("=== Wallet ===");
    println!("Network: regtest");
    println!("Receive address: {}", receive.address);
    println!("Change address:  {}", change.address);

    let recipient = "bcrt1qyslqw4ecxqgc5ng7cete7l8deax9tulvwem7f5";

let recipient_address = recipient
    .parse::<Address<_>>()?
    .require_network(Network::Regtest)?;

let mut tx_builder = wallet.build_tx();

tx_builder.add_recipient(
    recipient_address.script_pubkey(),
    Amount::from_btc(1.0)?,
);

let psbt = tx_builder.finish()?;

println!();
println!("=== Transaction ===");
println!("PSBT created successfully.");
println!("Inputs: {}", psbt.inputs.len());
println!("Outputs: {}", psbt.outputs.len());

    Ok(())
}