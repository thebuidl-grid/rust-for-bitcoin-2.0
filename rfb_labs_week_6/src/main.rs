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
    Ok(())
}
