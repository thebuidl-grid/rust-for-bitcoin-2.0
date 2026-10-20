use clap::Parser;
use decodetrx::{add_witness, build_transaction, parse_input, parse_output};

#[derive(Parser, Debug)]
#[command(
    name = "decodetrx",
    about = "Construct and serialize Bitcoin transactions"
)]
struct Args {
    /// Transaction version
    #[arg(short = 'v', long, default_value_t = 2)]
    version: u32,

    /// SegWit status (true or false)
    #[arg(long, default_value_t = false)]
    segwit: bool,

    /// Input: TXID:VOUT:SEQUENCE:SCRIPTSIG
    /// Repeat for multiple inputs.
    #[arg(short = 'i', long = "input", required = true)]
    inputs: Vec<String>,

    /// Output: AMOUNT_IN_SATOSHIS:SCRIPTPUBKEY
    /// Repeat for multiple outputs.
    #[arg(short = 'o', long = "output", required = true)]
    outputs: Vec<String>,

    /// Witness item: INPUT_INDEX:ITEM_HEX
    /// Repeat for multiple witness items.
    #[arg(short = 'w', long = "witness")]
    witnesses: Vec<String>,

    /// Transaction locktime
    #[arg(short = 'l', long, default_value_t = 0)]
    locktime: u32,
}

fn main() {
    let args = Args::parse();

    if let Err(error) = run(args) {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = Vec::new();

    for input in &args.inputs {
        inputs.push(parse_input(input)?);
    }

    let mut outputs = Vec::new();

    for output in &args.outputs {
        outputs.push(parse_output(output)?);
    }

    for witness in &args.witnesses {
        add_witness(&mut inputs, witness)?;
    }

    let has_witness = inputs.iter().any(|input| !input.witness.is_empty());

    if args.segwit && !has_witness {
        return Err("SegWit is enabled but no witness data was provided".into());
    }

    if !args.segwit && has_witness {
        return Err("witness data was provided but SegWit is disabled".into());
    }

    let (serialized, size, transaction) =
        build_transaction(args.version, inputs, outputs, args.locktime)?;

    println!("Transaction version: {}", transaction.version);
    println!("SegWit: {}", if has_witness { "yes" } else { "no" });
    println!("Transaction inputs: {}", transaction.inputs.len());
    println!("Transaction outputs: {}", transaction.outputs.len());
    println!(
        "Witness data: {}",
        if has_witness { "present" } else { "none" }
    );
    println!("Locktime: {}", transaction.lock_time);
    println!();
    println!("Serialized transaction:");
    println!("{}", serialized);
    println!();
    println!("Transaction size: {} bytes", size);

    Ok(())
}
