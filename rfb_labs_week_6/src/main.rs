use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::env;
use bdk_wallet::{KeychainKind, Wallet, PersistedWallet};
use bitcoin::{bip32::Xpriv, Network};
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::SignOptions;
use bitcoin::{Address, Amount};
use std::str::FromStr;
 use bdk_bitcoind_rpc::Emitter;


fn build_wallet(conn: &mut Connection) -> anyhow::Result<PersistedWallet<Connection>> {
    let xprv_str = env::var("WALLET_XPRIV").expect("WALLET_XPRIV must be set");
    let master: Xpriv = xprv_str.parse()?;
    //println!("WALLET_XPRIV={master}");

    let external = format!("wpkh({master}/84'/1'/0'/0/*)");
    let internal = format!("wpkh({master}/84'/1'/0'/1/*)");

    let wallet = match Wallet::load()
        .descriptor(KeychainKind::External, Some(external.clone()))
        .descriptor(KeychainKind::Internal, Some(internal.clone()))
        .extract_keys()
        .load_wallet(conn)?{
        Some(wallet) => wallet,
        None => Wallet::create(external, internal).network(Network::Regtest).create_wallet(conn)?,
    };

    Ok(wallet)
    
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let rpc_user = env::var("RPC_USER").expect("RPC_USER must be set");
    let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD must be set");
    
    let client = Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password))?;

    let info = client.get_blockchain_info()?;
    println!("chain: {}", info.chain);
    println!("block: {}", info.blocks);


    //calling the build_wallet function
    let mut conn = Connection::open("wallet.sqlite")?;
    let mut wallet = build_wallet(&mut conn)?;

    let external_address = wallet.next_unused_address(KeychainKind::External);
    let internal_address = wallet.next_unused_address(KeychainKind::Internal);

    println!("External Address: {}", external_address.address);
    println!("Internal Address: {}", internal_address.address);


    let mut emitter = Emitter::new(&client, wallet.latest_checkpoint(), 0, Vec::<bitcoin::Transaction>::new());

    while let Some(block_event) = emitter.next_block()?{
        wallet.apply_block(&block_event.block, block_event.block_height())?;
    }

    let mempool_event = emitter.mempool()?;
    wallet.apply_unconfirmed_txs(mempool_event.update);

    println!("balance: {}", wallet.balance().total());

    if let Ok(destination) = env::var("SEND_TO_ADDRESS") {
        let destination = Address::from_str(&destination)?.require_network(Network::Regtest)?;

        let mut builder = wallet.build_tx();
        builder.add_recipient(destination.script_pubkey(), Amount::from_sat(50_000));
        let mut psbt = builder.finish()?;

        for (i, input) in psbt.inputs.iter().enumerate() {
            println!(
                "input {i}: bip32_derivation = {:?}",
                input.bip32_derivation
            );
        }

        let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
        // println!("finalized: {finalized}");


        assert!(finalized, "wallet should be able to sign all its own inputs");


        let tx = psbt.extract_tx()?;

        let txid = client.send_raw_transaction(&tx)?;
        println!("broadcast txid: {txid}");
    }

    if let Some(changeset) = wallet.take_staged() {
        let db_tx = conn.transaction()?;
        changeset.persist_to_sqlite(&db_tx)?;
        db_tx.commit()?;
    }
     

    Ok(())
}
