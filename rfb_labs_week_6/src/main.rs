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
    println!("external descriptor: {}", descriptors.external);
    println!("internal descriptor: {}", descriptors.internal);

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

    let rpc_client = node::build_rpc_client(&config.rpc_url, &config.rpc_auth)?;
    let (chain, blocks) = node::chain_info(&rpc_client)?;
    println!("connected to node: chain={chain} blocks={blocks}");

    Ok(())
}
