use std::error::Error;

use clap::Parser;

use crate::transaction::{hex_to_bytes, Transaction, TxInput, TxOutput};

/// Build and serialize a Bitcoin transaction from command-line arguments.
#[derive(Parser, Debug)]
#[command(name = "serializetrx", about = "Serialize a Bitcoin transaction described on the command line")]
pub struct Cli {
    /// Transaction version (nVersion)
    #[arg(long, default_value_t = 2)]
    pub version: i32,

    /// Whether to encode this as a SegWit transaction (marker/flag + witness data)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub segwit: bool,

    /// Locktime (nLockTime)
    #[arg(long, default_value_t = 0)]
    pub locktime: u32,

    /// A transaction input: PREV_TXID:VOUT:SEQUENCE[:SCRIPT_SIG_HEX]. Repeat for multiple inputs.
    #[arg(long = "input", required = true)]
    pub inputs: Vec<String>,

    /// A transaction output: VALUE_SATS:SCRIPT_PUBKEY_HEX. Repeat for multiple outputs.
    #[arg(long = "output", required = true)]
    pub outputs: Vec<String>,

    /// A witness item for a given input: INPUT_INDEX:WITNESS_ITEM_HEX. Repeat as needed.
    #[arg(long = "witness")]
    pub witness: Vec<String>,
}

fn parse_sequence(raw: &str) -> Result<u32, Box<dyn Error>> {
    if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        Ok(u32::from_str_radix(hex, 16)?)
    } else {
        Ok(raw.parse::<u32>()?)
    }
}

fn parse_input(spec: &str) -> Result<TxInput, Box<dyn Error>> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 3 && parts.len() != 4 {
        return Err(format!(
            "invalid --input '{spec}': expected PREV_TXID:VOUT:SEQUENCE[:SCRIPT_SIG_HEX]"
        )
        .into());
    }

    let prev_txid = hex_to_bytes(parts[0])
        .map_err(|e| format!("invalid --input '{spec}': bad PREV_TXID hex: {e}"))?;
    if prev_txid.len() != 32 {
        return Err(format!(
            "invalid --input '{spec}': PREV_TXID must be 32 bytes (64 hex chars), got {}",
            prev_txid.len()
        )
        .into());
    }

    let vout: u32 = parts[1]
        .parse()
        .map_err(|e| format!("invalid --input '{spec}': bad VOUT: {e}"))?;

    let sequence = parse_sequence(parts[2])
        .map_err(|e| format!("invalid --input '{spec}': bad SEQUENCE: {e}"))?;

    let script_sig = match parts.get(3) {
        Some(hex) => hex_to_bytes(hex)
            .map_err(|e| format!("invalid --input '{spec}': bad SCRIPT_SIG hex: {e}"))?,
        None => vec![],
    };

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness: vec![],
    })
}

fn parse_output(spec: &str) -> Result<TxOutput, Box<dyn Error>> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid --output '{spec}': expected VALUE_SATS:SCRIPT_PUBKEY_HEX"
        )
        .into());
    }

    let value: u64 = parts[0]
        .parse()
        .map_err(|e| format!("invalid --output '{spec}': bad VALUE_SATS: {e}"))?;

    let script_pubkey = hex_to_bytes(parts[1])
        .map_err(|e| format!("invalid --output '{spec}': bad SCRIPT_PUBKEY hex: {e}"))?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

fn parse_witness(spec: &str, input_count: usize) -> Result<(usize, Vec<u8>), Box<dyn Error>> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid --witness '{spec}': expected INPUT_INDEX:WITNESS_ITEM_HEX"
        )
        .into());
    }

    let input_index: usize = parts[0]
        .parse()
        .map_err(|e| format!("invalid --witness '{spec}': bad INPUT_INDEX: {e}"))?;
    if input_index >= input_count {
        return Err(format!(
            "invalid --witness '{spec}': INPUT_INDEX {input_index} out of range, only {input_count} input(s) provided"
        )
        .into());
    }

    let item = hex_to_bytes(parts[1])
        .map_err(|e| format!("invalid --witness '{spec}': bad WITNESS_ITEM hex: {e}"))?;

    Ok((input_index, item))
}

pub fn build_transaction(cli: &Cli) -> Result<Transaction, Box<dyn Error>> {
    let mut inputs: Vec<TxInput> = cli
        .inputs
        .iter()
        .map(|spec| parse_input(spec))
        .collect::<Result<_, _>>()?;

    let outputs: Vec<TxOutput> = cli
        .outputs
        .iter()
        .map(|spec| parse_output(spec))
        .collect::<Result<_, _>>()?;

    for spec in &cli.witness {
        let (input_index, item) = parse_witness(spec, inputs.len())?;
        inputs[input_index].witness.push(item);
    }

    Ok(Transaction {
        version: cli.version,
        inputs,
        outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    })
}
