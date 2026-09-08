use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "serialize-trx",
    version,
    about = "Serialize a Bitcoin transaction into raw hexadecimal"
)]
pub struct Cli {
    /// Transaction version number.
    #[arg(long)]
    pub tx_version: i32,

    /// Transaction locktime as a block height or Unix timestamp.
    #[arg(long)]
    pub locktime: u32,
}
