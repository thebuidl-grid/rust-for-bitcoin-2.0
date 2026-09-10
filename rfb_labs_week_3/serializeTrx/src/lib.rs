pub mod error;
pub mod serializer;
pub mod transaction;

pub use error::TxSerializerError;
pub use serializer::{bytes_to_hex, hex_to_bytes, serialize_transaction};
pub use transaction::{Transaction, TxInput, TxOutput};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "Bitcoin Transaction Serializer",
    version = "0.1.0",
    about = "Serialize Bitcoin transactions from command-line arguments",
    long_about = "Serialize Bitcoin transactions without hardcoding transaction data. Supports both legacy and SegWit transaction formats."
)]
pub struct Args {
    /// Transaction version (default: 2)
    #[arg(short = 't', long, default_value = "2")]
    pub tx_version: i32,

    /// Enable SegWit transaction format
    #[arg(short = 's', long)]
    pub segwit: bool,

    /// Transaction input in format: prev_txid:vout[:script_sig[:sequence]]
    /// - prev_txid: 64 hex characters (32 bytes, reversed from block explorer)
    /// - vout: output index as decimal number
    /// - script_sig: hex string (optional, default empty)
    /// - sequence: hex string (optional, default 0xffffffff)
    #[arg(short = 'i', long)]
    pub input: Vec<String>,

    /// Transaction output in format: value:script_pubkey
    /// - value: satoshis as decimal number
    /// - script_pubkey: hex string
    #[arg(short = 'o', long)]
    pub output: Vec<String>,

    /// Witness item as hex string (for SegWit)
    /// Use multiple times for multiple witness items
    #[arg(short = 'w', long)]
    pub witness: Vec<String>,

    /// Witness items per input (comma-separated counts)
    /// Example: --witness-counts 2,1 for 2 inputs with 2 and 1 witness items
    #[arg(long)]
    pub witness_counts: Option<String>,

    /// Locktime in blocks or timestamp (default: 0)
    #[arg(short = 'l', long, default_value = "0")]
    pub locktime: u32,
}

pub fn run(args: Args) -> Result<String, TxSerializerError> {
    // Validate inputs and outputs
    if args.input.is_empty() {
        return Err(TxSerializerError::ArgError(
            "At least one input required (-i/--input)".to_string(),
        ));
    }

    if args.output.is_empty() {
        return Err(TxSerializerError::ArgError(
            "At least one output required (-o/--output)".to_string(),
        ));
    }

    // Parse inputs
    let mut inputs = Vec::new();
    for (idx, input_str) in args.input.iter().enumerate() {
        inputs.push(parse_input(input_str, idx)?);
    }

    // Parse outputs
    let mut outputs = Vec::new();
    for output_str in &args.output {
        outputs.push(parse_output(output_str)?);
    }

    // Assign witness data to inputs if SegWit enabled
    if args.segwit {
        assign_witness_data(&mut inputs, &args.witness, &args.witness_counts)?;
    } else if !args.witness.is_empty() {
        return Err(TxSerializerError::ArgError(
            "Witness data provided but SegWit flag not enabled (-s/--segwit)".to_string(),
        ));
    }

    // Create transaction
    let transaction =
        Transaction::new(args.tx_version, inputs, outputs, args.locktime, args.segwit)?;

    // Serialize transaction
    let serialized = serialize_transaction(&transaction);
    let hex_output = bytes_to_hex(&serialized);

    // Format output
    let mut result = String::new();
    result.push_str("\n╔═══════════════════════════════════════════════════════════════╗\n");
    result.push_str("║        Bitcoin Transaction Serialization Result               ║\n");
    result.push_str("╚═══════════════════════════════════════════════════════════════╝\n\n");

    result.push_str("Transaction Details:\n");
    result.push_str("─────────────────────────────────────────────────────────────\n");
    result.push_str(&format!(
        "  Version:                  {}\n",
        transaction.version
    ));
    result.push_str(&format!(
        "  SegWit Enabled:           {}\n",
        transaction.segwit
    ));
    result.push_str(&format!(
        "  Number of Inputs:         {}\n",
        transaction.inputs.len()
    ));
    result.push_str(&format!(
        "  Number of Outputs:        {}\n",
        transaction.outputs.len()
    ));
    result.push_str(&format!(
        "  Locktime:                 {}\n\n",
        transaction.locktime
    ));

    result.push_str("Input Details:\n");
    result.push_str("─────────────────────────────────────────────────────────────\n");
    for (idx, input) in transaction.inputs.iter().enumerate() {
        result.push_str(&format!("  Input {}\n", idx));
        result.push_str(&format!("    Previous TXID:   {}\n", input.get_txid_hex()));
        result.push_str(&format!("    VOUT:            {}\n", input.vout));
        result.push_str(&format!(
            "    ScriptSig:       {}\n",
            if input.script_sig.is_empty() {
                "(empty)".to_string()
            } else {
                bytes_to_hex(&input.script_sig)
            }
        ));
        result.push_str(&format!("    Sequence:        0x{:08x}\n", input.sequence));

        if !input.witness.is_empty() {
            result.push_str(&format!("    Witness Items:   {}\n", input.witness.len()));
            for (w_idx, witness_item) in input.witness.iter().enumerate() {
                let item_hex = bytes_to_hex(witness_item);
                let display_hex = if item_hex.len() > 40 {
                    format!("{}...", &item_hex[..40])
                } else {
                    item_hex
                };
                result.push_str(&format!("      [{0}] {1}\n", w_idx, display_hex));
            }
        }
        result.push_str("\n");
    }

    result.push_str("Output Details:\n");
    result.push_str("─────────────────────────────────────────────────────────────\n");
    for (idx, output) in transaction.outputs.iter().enumerate() {
        result.push_str(&format!("  Output {}\n", idx));
        result.push_str(&format!("    Value:           {} satoshis\n", output.value));
        result.push_str(&format!(
            "    ScriptPubKey:    {}\n",
            output.get_script_hex()
        ));
        result.push_str("\n");
    }

    result.push_str("Serialization Results:\n");
    result.push_str("─────────────────────────────────────────────────────────────\n");
    result.push_str(&format!(
        "  Transaction Size: {} bytes\n\n",
        serialized.len()
    ));
    result.push_str("Serialized Transaction (Hex):\n");
    result.push_str("─────────────────────────────────────────────────────────────\n");
    result.push_str(&hex_output);
    result.push_str("\n\n");

    Ok(result)
}

pub fn parse_input(input_str: &str, idx: usize) -> Result<TxInput, TxSerializerError> {
    let parts: Vec<&str> = input_str.split(':').collect();

    if parts.len() < 2 {
        return Err(TxSerializerError::InvalidInputFormat(format!(
            "Input {}: {}",
            idx, input_str
        )));
    }

    // Parse prev_txid (must be 64 hex characters = 32 bytes)
    let prev_txid = hex_to_bytes(parts[0])?;
    if prev_txid.len() != 32 {
        return Err(TxSerializerError::InvalidTxIdLength(prev_txid.len()));
    }

    // Parse vout
    let vout: u32 = parts[1]
        .parse()
        .map_err(|_| TxSerializerError::InvalidVout(parts[1].to_string()))?;

    // Parse script_sig (optional, default empty)
    let script_sig = if parts.len() > 2 && !parts[2].is_empty() {
        hex_to_bytes(parts[2])?
    } else {
        Vec::new()
    };

    // Parse sequence (optional, default 0xffffffff)
    let sequence = if parts.len() > 3 && !parts[3].is_empty() {
        u32::from_str_radix(parts[3], 16)
            .map_err(|_| TxSerializerError::InvalidSequence(parts[3].to_string()))?
    } else {
        0xffffffff
    };

    TxInput::new(prev_txid, vout, script_sig, sequence)
}

pub fn parse_output(output_str: &str) -> Result<TxOutput, TxSerializerError> {
    let parts: Vec<&str> = output_str.split(':').collect();

    if parts.len() != 2 {
        return Err(TxSerializerError::InvalidOutputFormat(
            output_str.to_string(),
        ));
    }

    // Parse value (satoshis)
    let value: u64 = parts[0]
        .parse()
        .map_err(|_| TxSerializerError::InvalidValue(parts[0].to_string()))?;

    // Parse script_pubkey
    let script_pubkey = hex_to_bytes(parts[1])?;

    Ok(TxOutput::new(value, script_pubkey))
}

pub fn assign_witness_data(
    inputs: &mut [TxInput],
    witness_items: &[String],
    witness_counts: &Option<String>,
) -> Result<(), TxSerializerError> {
    if witness_items.is_empty() {
        return Err(TxSerializerError::ArgError(
            "SegWit enabled but no witness data provided (-w/--witness)".to_string(),
        ));
    }

    // Parse witness counts if provided
    let counts = if let Some(counts_str) = witness_counts {
        let mut parsed_counts = Vec::new();
        for count_str in counts_str.split(',') {
            let count: usize = count_str.trim().parse().map_err(|_| {
                TxSerializerError::InvalidWitnessCountFormat(counts_str.to_string())
            })?;
            parsed_counts.push(count);
        }
        parsed_counts
    } else {
        // Default: distribute witness items evenly across inputs
        let items_per_input = (witness_items.len() + inputs.len() - 1) / inputs.len();
        vec![items_per_input; inputs.len()]
    };

    // Validate counts match number of inputs
    if counts.len() != inputs.len() {
        return Err(TxSerializerError::ArgError(format!(
            "Witness count mismatch: expected {} counts, got {}",
            inputs.len(),
            counts.len()
        )));
    }

    // Distribute witness items
    let mut witness_idx = 0;
    for (input_idx, input) in inputs.iter_mut().enumerate() {
        let count = counts[input_idx];

        if witness_idx + count > witness_items.len() {
            return Err(TxSerializerError::ArgError(format!(
                "Not enough witness items. Input {} needs {} items, but only {} remaining",
                input_idx,
                count,
                witness_items.len() - witness_idx
            )));
        }

        for _ in 0..count {
            let witness_hex = &witness_items[witness_idx];
            let witness_bytes = hex_to_bytes(witness_hex)?;
            input.add_witness_item(witness_bytes);
            witness_idx += 1;
        }
    }

    // Check if all witness items were used
    if witness_idx < witness_items.len() {
        return Err(TxSerializerError::ArgError(format!(
            "Unused witness items: {} provided but only {} used",
            witness_items.len(),
            witness_idx
        )));
    }

    Ok(())
}
