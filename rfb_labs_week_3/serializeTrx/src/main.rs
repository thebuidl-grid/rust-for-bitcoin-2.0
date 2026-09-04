use clap::{ArgAction, Parser};
use serializetrx::{
    SerializeError, TxidOrder, build_transaction, bytes_to_hex, parse_input, parse_output,
    parse_witness, serialize_transaction,
};

#[derive(Parser)]
// No `version` attribute here on purpose: it would generate a `--version`
// flag for the program itself and collide with the transaction's own
// `--version` field below, which is the one the user actually needs.
#[command(
    name = "serializetrx",
    about = "Builds and serializes a Bitcoin transaction from command-line values",
    after_help = "\
SPEC FORMATS
  --input     txid:vout[:script_sig_hex[:sequence]]
              script_sig defaults to empty, sequence to 4294967295 (0xffffffff)
  --output    amount_sats:script_pubkey_hex
  --witness   input_index:item_hex[,item_hex...]

  Numbers accept decimal or a 0x hex prefix.
  txids are taken in explorer (display) order by default; pass
  --txid-order internal for values already in wire order.

EXAMPLE
  serializetrx --segwit \\
    --input bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1 \\
    --output 100:0014274ae586ad2035efb4c25049c155f98310d7e106 \\
    --output 4462282:0014599bcef6387256c6b019030c421b4a4d382fe260 \\
    --witness 0:3044...01,020cb9...f1"
)]
struct Cli {
    /// Transaction version
    #[arg(long, default_value_t = 2)]
    version: i32,

    /// Mark the transaction as SegWit (adds the BIP144 marker and flag)
    #[arg(long, action = ArgAction::SetTrue)]
    segwit: bool,

    /// Transaction input: txid:vout[:script_sig_hex[:sequence]] (repeatable)
    #[arg(long, required = true, value_name = "SPEC")]
    input: Vec<String>,

    /// Transaction output: amount_sats:script_pubkey_hex (repeatable)
    #[arg(long, required = true, value_name = "SPEC")]
    output: Vec<String>,

    /// Witness data: input_index:item_hex[,item_hex...] (repeatable)
    #[arg(long, value_name = "SPEC")]
    witness: Vec<String>,

    /// Transaction locktime
    #[arg(long, default_value_t = 0)]
    locktime: u32,

    /// Byte order of the txids given to --input
    #[arg(long, value_name = "ORDER", default_value = "display",
          value_parser = ["display", "internal"])]
    txid_order: String,
}

fn run(cli: Cli) -> Result<(), SerializeError> {
    let order = match cli.txid_order.as_str() {
        "internal" => TxidOrder::Internal,
        _ => TxidOrder::Display,
    };

    let inputs = cli
        .input
        .iter()
        .map(|spec| parse_input(spec, order))
        .collect::<Result<Vec<_>, _>>()?;

    let outputs = cli
        .output
        .iter()
        .map(|spec| parse_output(spec))
        .collect::<Result<Vec<_>, _>>()?;

    let witnesses = cli
        .witness
        .iter()
        .map(|spec| parse_witness(spec))
        .collect::<Result<Vec<_>, _>>()?;

    let transaction = build_transaction(
        cli.version,
        cli.segwit,
        inputs,
        outputs,
        witnesses,
        cli.locktime,
    )?;

    let serialized = serialize_transaction(&transaction);

    println!("Serialized transaction (hex):");
    println!("{}", bytes_to_hex(&serialized));
    println!();
    println!("Transaction size: {} bytes", serialized.len());

    Ok(())
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("error: {}", error);
        std::process::exit(1);
    }
}
