use std::sync::Arc;

mod config;
mod error;
mod keys;
mod node;
mod persist;
mod tx;
mod wallet;

fn main() -> anyhow::Result<()> {
    let config = config::Config::from_env()?;
    println!("{config:?}");

    let mnemonic = keys::load_or_generate_mnemonic(config.mnemonic.as_deref())?;
    let descriptors =
        keys::descriptors_from_mnemonic(&mnemonic, config.network.into())?;
    println!("external descriptor (public): {}", descriptors.external_public);
    println!("internal descriptor (public): {}", descriptors.internal_public);

    let mut db = persist::open_db(&config.db_path)?;
    let mut w = wallet::open_or_create_wallet(
        descriptors.external,
        descriptors.internal,
        config.network,
        &mut db,
    )?;

    let receive = wallet::new_receive_address(&mut w);
    let change = wallet::new_change_address(&mut w);
    w.persist(&mut db)?;

    println!("receive address (external): {}", receive.address);
    println!("change address (internal):  {}", change.address);
    assert_ne!(receive.address, change.address);

    let rpc_client = Arc::new(node::build_rpc_client(&config.rpc_url, &config.rpc_auth)?);
    let (chain, blocks) = node::chain_info(&rpc_client)?;
    println!("connected to node: chain={chain} blocks={blocks}");

    if chain == "regtest" {
        let mined = node::fund_wallet_regtest(
            &rpc_client,
            &receive.address,
            node::COINBASE_MATURITY + 1,
        )?;
        println!("mined {} blocks to {}", mined.len(), receive.address);
    }

    node::sync_wallet(&mut w, Arc::clone(&rpc_client), &mut db)?;
    let balance = wallet::get_balance(&w);
    let utxos = wallet::list_utxos(&w);
    println!(
        "balance: total={} confirmed={} trusted_pending={}",
        balance.total(),
        balance.confirmed,
        balance.trusted_pending
    );
    println!("utxos: {}", utxos.len());

    if balance.confirmed > bitcoin::Amount::ZERO {
        let send_to = wallet::new_receive_address(&mut w);
        w.persist(&mut db)?;

        let txid = tx::send(
            &mut w,
            &rpc_client,
            &mut db,
            &send_to.address,
            bitcoin::Amount::from_btc(1.0).expect("valid amount"),
            bitcoin::FeeRate::from_sat_per_vb(2).expect("valid fee rate"),
        )?;
        println!("broadcast txid: {txid}");

        if chain == "regtest" {
            node::fund_wallet_regtest(&rpc_client, &receive.address, 1)?;
            node::sync_wallet(&mut w, Arc::clone(&rpc_client), &mut db)?;
            let balance = wallet::get_balance(&w);
            println!(
                "balance after 1 confirmation: total={} confirmed={}",
                balance.total(),
                balance.confirmed
            );
        }
    } else {
        println!("no confirmed balance yet — skipping send demo");
    }

    Ok(())
}
