mod transaction;

use std::env;
use transaction::{
    bytes_to_hex, hex_to_bytes, serialize_transaction, validate_txid, Transaction, TxInput,
    TxOutput,
};

const USAGE: &str = "\
serializeTrx - build and serialize a raw Bitcoin transaction from CLI flags

USAGE:
    serializeTrx --version <i32> [--segwit] --input <spec> [--input <spec>]... \\
                 --output <spec> [--output <spec>]... [--witness <spec>]... \\
                 --locktime <u32>

FLAGS:
    -h, --help              Print this help message and exit

OPTIONS:
    --version <i32>         Transaction version (required)
    --segwit                Mark transaction as segwit; enables witness serialization
    --input <spec>           txid_hex:vout[:sequence[:scriptsig_hex]]  (repeatable, >=1 required)
    --output <spec>          value_sats:scriptpubkey_hex               (repeatable, >=1 required)
    --witness <spec>         input_index:item_hex                      (repeatable, requires --segwit)
    --locktime <u32>         Locktime (required)

NOTES:
    - All hex fields accept upper or lower case and must have an even
      number of characters.
    - --input's optional trailing fields (sequence, scriptsig_hex) default
      to 0xffffffff and an empty script respectively. A field may be left
      blank to skip it while still supplying a later one, e.g.
      'txid:1::deadbeef' skips sequence but sets scriptsig_hex.
    - --witness <input_index>:<item_hex> attaches a witness item to the
      input at that 0-based index (in the order --input flags were given).
      Multiple --witness flags for the same index are appended in order.
      Using --witness requires --segwit to also be set.

EXAMPLES:
    cargo run -- \
    --version 2 \
    --segwit \
    --input ce243bc6c7bdfd8ce98161fd8e51515c5310480e3431ea415becf900609d0f31:0 \
    --witness 0:304402203c4bbaf5ddf40d332ce1c98950e8f15ea073f549817a08abcf33fdf8586e527902200a4e99d28894213c0feb4eb78808b1942ff8dceba9fc3836f0f20b40483b624201 \
    --witness 0:03d343042aeebc549ecc8b594d647261cb976b3ccc3faa23027785e66547507b9d \
    --output 6120:00147d146a2f5a1db27c9b7288350fe804b258f6c6b0 \
    --locktime 0
";

struct ParsedArgs {
    version: Option<i32>,
    segwit: bool,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    witness_entries: Vec<(usize, Vec<u8>)>,
    locktime: Option<u32>,
}

fn main() {
    let raw_args: Vec<String> = env::args().collect();

    if raw_args.iter().skip(1).any(|a| a == "-h" || a == "--help") {
        print!("{}", USAGE);
        return;
    }

    if let Err(e) = run(&raw_args[1..]) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let parsed = parse_args(args)?;
    let trx = build_transaction(parsed)?;
    let serialized = serialize_transaction(&trx);

    println!("Serialized transaction (hex):");
    println!("{}", bytes_to_hex(&serialized));
    println!("\nTransaction size: {} bytes", serialized.len());

    Ok(())
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut parsed = ParsedArgs {
        version: None,
        segwit: false,
        inputs: Vec::new(),
        outputs: Vec::new(),
        witness_entries: Vec::new(),
        locktime: None,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                let val = next_value(args, &mut i, "--version")?;
                parsed.version = Some(
                    val.parse::<i32>()
                        .map_err(|_| format!("invalid --version value '{}': must be a valid i32 integer", val))?,
                );
            }
            "--segwit" => {
                parsed.segwit = true;
                i += 1;
            }
            "--input" => {
                let val = next_value(args, &mut i, "--input")?;
                parsed.inputs.push(parse_input_spec(val)?);
            }
            "--output" => {
                let val = next_value(args, &mut i, "--output")?;
                parsed.outputs.push(parse_output_spec(val)?);
            }
            "--witness" => {
                let val = next_value(args, &mut i, "--witness")?;
                parsed.witness_entries.push(parse_witness_spec(val)?);
            }
            "--locktime" => {
                let val = next_value(args, &mut i, "--locktime")?;
                parsed.locktime = Some(
                    val.parse::<u32>()
                        .map_err(|_| format!("invalid --locktime value '{}': must be a valid u32 integer", val))?,
                );
            }
            other => {
                return Err(format!("unknown argument '{}'; run with --help for usage", other));
            }
        }
    }

    Ok(parsed)
}

fn next_value<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
    let value = args
        .get(*i + 1)
        .ok_or_else(|| format!("{} requires a value", flag))?;
    *i += 2;
    Ok(value.as_str())
}

fn parse_input_spec(spec: &str) -> Result<TxInput, String> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return Err(format!(
            "invalid --input '{}': expected format txid_hex:vout[:sequence[:scriptsig_hex]]",
            spec
        ));
    }

    let txid_hex = parts[0];
    let prev_txid = hex_to_bytes(txid_hex)
        .map_err(|e| format!("invalid hex in --input txid: '{}' ({})", txid_hex, e))?;
    validate_txid(&prev_txid)
        .map_err(|e| format!("invalid --input txid '{}': {}", txid_hex, e))?;

    let vout: u32 = parts[1]
        .parse()
        .map_err(|_| format!("invalid --input vout '{}': must be a valid u32 integer", parts[1]))?;

    let sequence: u32 = match parts.get(2) {
        Some(s) if !s.is_empty() => s
            .parse()
            .map_err(|_| format!("invalid --input sequence '{}': must be a valid u32 integer", s))?,
        _ => 0xffffffff,
    };

    let script_sig = match parts.get(3) {
        Some(s) if !s.is_empty() => hex_to_bytes(s)
            .map_err(|e| format!("invalid hex in --input scriptsig: '{}' ({})", s, e))?,
        _ => Vec::new(),
    };

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness: Vec::new(),
    })
}

fn parse_output_spec(spec: &str) -> Result<TxOutput, String> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid --output '{}': expected format value_sats:scriptpubkey_hex",
            spec
        ));
    }

    let value: u64 = parts[0].parse().map_err(|_| {
        format!(
            "invalid --output value '{}': must be a valid u64 integer (satoshis)",
            parts[0]
        )
    })?;

    let script_pubkey = hex_to_bytes(parts[1])
        .map_err(|e| format!("invalid hex in --output scriptpubkey: '{}' ({})", parts[1], e))?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

fn parse_witness_spec(spec: &str) -> Result<(usize, Vec<u8>), String> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid --witness '{}': expected format input_index:item_hex",
            spec
        ));
    }

    let input_index: usize = parts[0].parse().map_err(|_| {
        format!(
            "invalid --witness input_index '{}': must be a non-negative integer",
            parts[0]
        )
    })?;

    let item = hex_to_bytes(parts[1])
        .map_err(|e| format!("invalid hex in --witness item: '{}' ({})", parts[1], e))?;

    Ok((input_index, item))
}

fn build_transaction(mut parsed: ParsedArgs) -> Result<Transaction, String> {
    let version = parsed
        .version
        .ok_or_else(|| "missing required argument: --version".to_string())?;
    let locktime = parsed
        .locktime
        .ok_or_else(|| "missing required argument: --locktime".to_string())?;

    if parsed.inputs.is_empty() {
        return Err("at least one --input is required".to_string());
    }
    if parsed.outputs.is_empty() {
        return Err("at least one --output is required".to_string());
    }
    if !parsed.witness_entries.is_empty() && !parsed.segwit {
        return Err(
            "--witness was given but --segwit was not set; add --segwit or remove the --witness flags"
                .to_string(),
        );
    }

    let input_count = parsed.inputs.len();
    for (idx, item) in parsed.witness_entries {
        let input = parsed.inputs.get_mut(idx).ok_or_else(|| {
            format!(
                "--witness refers to input index {} but only {} input(s) were provided (valid range: 0..={})",
                idx,
                input_count,
                input_count.saturating_sub(1)
            )
        })?;
        input.witness.push(item);
    }

    Ok(Transaction {
        version,
        inputs: parsed.inputs,
        outputs: parsed.outputs,
        locktime,
        segwit: parsed.segwit,
    })
}
