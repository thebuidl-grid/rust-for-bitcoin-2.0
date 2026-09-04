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

    Ok(())
}
