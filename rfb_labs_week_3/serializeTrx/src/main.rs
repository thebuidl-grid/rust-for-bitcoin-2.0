use std::error::Error;

use clap::Parser;

mod cli;
mod encoding;
mod error;
mod transaction;

use cli::Cli;
use encoding::{bytes_to_hex, hex_to_bytes};
use transaction::{Transaction, TxInput, TxOutput, serialize_transaction};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let input = TxInput {
        prev_txid: hex_to_bytes(
            "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
        )?,
        vout: 1,
        script_sig: vec![],
        sequence: 0xffffffff,
        witness: vec![
            hex_to_bytes(
                "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301",
            )?,
            hex_to_bytes("029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358")?,
        ],
    };

    let output_0 = TxOutput {
        value: 69886,
        script_pubkey: hex_to_bytes("0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b")?,
    };

    let output_1 = TxOutput {
        value: 29442,
        script_pubkey: hex_to_bytes("00149831122b93d21715c70db626ccc844d3c21f9687")?,
    };

    let trx = Transaction {
        version: cli.tx_version,
        inputs: vec![input],
        outputs: vec![output_0, output_1],
        locktime: cli.locktime,
    };

    // Serialize
    let serialized = serialize_transaction(&trx)?;

    println!("Serialized transaction:");
    println!("{:?}", &serialized);
    println!("Serialized Hex transaction:");
    println!("{}", bytes_to_hex(&serialized));

    println!("\nTransaction size: {} bytes", serialized.len());

    Ok(())
}
