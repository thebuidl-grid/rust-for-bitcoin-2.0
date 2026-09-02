//! The command line: what a user can say, and how it becomes a [`Transaction`].
//!
//! Nothing about the transaction is baked into the program any more. Every
//! field arrives as text, and everything in this module exists to turn that
//! text into bytes, or to explain why it cannot.

use clap::{Parser, ValueEnum};

use crate::error::{Result, TxError};
use crate::hex::{hex_to_bytes, hex_to_txid};
use crate::transaction::{Transaction, TxInput, TxOutput};

/// 21,000,000 BTC in satoshis: no output may exceed it.
const MAX_MONEY: i128 = 21_000_000 * 100_000_000;

const INPUT_KEYS: &str = "txid, vout, script_sig, sequence, witness";
const OUTPUT_KEYS: &str = "amount, script_pubkey";

const EXAMPLES: &str = "\
EXAMPLES:
  # A one input, two output P2WPKH spend
  serializetrx --version 2 \\
    --input txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1,witness=3045...01|029c...58 \\
    --output 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \\
    --output 29442:00149831122b93d21715c70db626ccc844d3c21f9687 \\
    --locktime 0

  # The same thing with the outpoint shorthand and the witness given by index
  serializetrx --input <txid>:1,sequence=0xfffffffd --witness 0:3045...01|029c...58 \\
    --output 69886:0014...

  # A legacy transaction: no witness anywhere, so no marker, flag or --segwit
  serializetrx --version 1 \\
    --input txid=<txid>,vout=0,script_sig=47304402... \\
    --output 5000000000:4104ae1a62fe...ac

SPEC FORMATS:
  --input    txid:vout
             txid=<64 hex>,vout=<n>[,script_sig=<hex>][,sequence=<n>][,witness=<hex>|<hex>]
             the two may be mixed: txid:vout,sequence=0xfffffffd,witness=<hex>|<hex>
  --output   <satoshis>:<script_pubkey hex>
             amount=<satoshis>,script_pubkey=<hex>
  --witness  <input index>:<hex>|<hex>        (indexes start at 0)

  Numbers may be decimal or 0x hexadecimal, and may contain _ separators.
";

/// Which way round the previous txid is written on the command line.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum TxidOrder {
    /// The order block explorers and `bitcoin-cli` show, i.e. reversed.
    /// Reversed again on the way into the transaction. This is the default.
    Display,
    /// Already in internal order: written into the transaction untouched.
    Internal,
}

#[derive(Parser, Debug)]
#[command(
    name = "serializetrx",
    about = "Serialise a Bitcoin transaction from values given on the command line",
    disable_version_flag = true,
    after_help = EXAMPLES
)]
pub struct Cli {
    /// Transaction version (also accepted as --tx-version)
    #[arg(
        long = "version",
        alias = "tx-version",
        value_name = "N",
        default_value = "2"
    )]
    pub version: String,

    /// An input: `txid:vout`, optionally followed by key=value fields. Repeat for more inputs
    #[arg(long, short, value_name = "SPEC", required = true)]
    pub input: Vec<String>,

    /// An output: `satoshis:script_pubkey`, or a comma separated list of keys. Repeat for more outputs
    #[arg(long, short, value_name = "SPEC", required = true)]
    pub output: Vec<String>,

    /// A witness stack for one input: `index:item|item`. Alternative to `witness=` inside --input
    #[arg(long, short, value_name = "INDEX:ITEMS")]
    pub witness: Vec<String>,

    /// Locktime: 0 for none, below 500000000 a block height, otherwise a Unix time
    #[arg(long, short, value_name = "N", default_value = "0")]
    pub locktime: String,

    /// Force the BIP144 SegWit serialisation (marker, flag and witness section)
    #[arg(long, conflicts_with = "no_segwit")]
    pub segwit: bool,

    /// Force the legacy serialisation. Without either flag, the format follows the witness data
    #[arg(long = "no-segwit")]
    pub no_segwit: bool,

    /// Byte order of the txid in --input
    #[arg(long, value_name = "ORDER", value_enum, default_value_t = TxidOrder::Display)]
    pub txid_order: TxidOrder,

    /// Also print a field by field breakdown of the transaction
    #[arg(long, short)]
    pub verbose: bool,

    /// Also print the serialised transaction as a decimal byte array
    #[arg(long)]
    pub bytes: bool,
}

/// Turn parsed command line text into a transaction, validating as it goes.
pub fn build_transaction(cli: &Cli) -> Result<Transaction> {
    let version = parse_i32("--version", &cli.version)?;
    let locktime = parse_u32("--locktime", &cli.locktime)?;

    // Inputs first: --witness refers to them by index, so the count has to be
    // known before those can be checked.
    let mut inputs = Vec::with_capacity(cli.input.len());
    // Which inputs already carry a stack, so a second one can be refused. An
    // inline `witness=` with nothing after it counts: an empty stack is a
    // deliberate choice, not an absence.
    let mut has_witness = Vec::with_capacity(cli.input.len());
    for (position, spec) in cli.input.iter().enumerate() {
        let (input, witness_given) = parse_input(position, spec, cli.txid_order)?;
        inputs.push(input);
        has_witness.push(witness_given);
    }

    let mut outputs = Vec::with_capacity(cli.output.len());
    for (position, spec) in cli.output.iter().enumerate() {
        outputs.push(parse_output(position, spec)?);
    }

    for spec in &cli.witness {
        let (index_text, items) = spec.split_once(':').ok_or_else(|| TxError::MalformedSpec {
            field: "--witness".to_string(),
            value: spec.clone(),
            expected: "an input index and a stack, such as 0:3045...01|029c...58",
        })?;

        let index = parse_index("--witness input index", index_text.trim())?;
        if index >= inputs.len() {
            return Err(TxError::WitnessIndex {
                index,
                inputs: inputs.len(),
            });
        }
        if has_witness[index] {
            return Err(TxError::WitnessConflict { index });
        }

        inputs[index].witness = parse_witness_items(&format!("--witness {index}"), items)?;
        has_witness[index] = true;
    }

    let witness_items: usize = inputs.iter().map(|input| input.witness.len()).sum();

    // With neither flag the format follows the data: a transaction carrying
    // witness items needs the SegWit serialisation, one without it does not.
    let segwit = if cli.segwit {
        true
    } else if cli.no_segwit {
        false
    } else {
        witness_items > 0
    };

    if !segwit && witness_items > 0 {
        return Err(TxError::WitnessWithoutSegwit {
            items: witness_items,
        });
    }
    if segwit && witness_items == 0 {
        return Err(TxError::SegwitWithoutWitness);
    }

    Ok(Transaction {
        version,
        inputs,
        outputs,
        locktime,
        segwit,
    })
}

/// Read one `--input`. Returns the input and whether a witness was given here,
/// which decides later whether a `--witness` for the same index is a clash.
fn parse_input(position: usize, spec: &str, order: TxidOrder) -> Result<(TxInput, bool)> {
    let field = format!("--input #{}", position + 1);

    let mut txid: Option<Vec<u8>> = None;
    let mut vout: Option<u32> = None;
    let mut script_sig = Vec::new();
    let mut sequence = 0xffff_ffff_u32;
    let mut witness = Vec::new();
    let mut witness_given = false;

    let mut seen: Vec<&str> = Vec::new();
    for (field_position, field_spec) in spec.split(',').enumerate() {
        let field_spec = field_spec.trim();
        if field_spec.is_empty() {
            continue;
        }

        // The first field may be the outpoint shorthand used by explorers and
        // bitcoin-cli, with anything else that input needs given as key=value
        // after it: `<txid>:0,sequence=0xfffffffd`.
        if field_position == 0 && !field_spec.contains('=') {
            let (txid_text, vout_text) =
                field_spec
                    .rsplit_once(':')
                    .ok_or_else(|| TxError::MalformedSpec {
                        field: field.clone(),
                        value: field_spec.to_string(),
                        expected: "txid:vout, optionally followed by key=value pairs such as \
                                   sequence=0xfffffffd",
                    })?;
            txid = Some(read_txid(&field, txid_text.trim(), order)?);
            vout = Some(parse_u32(&format!("{field} vout"), vout_text.trim())?);
            seen.push("txid");
            seen.push("vout");
            continue;
        }

        let (key, value) = field_spec
            .split_once('=')
            .ok_or_else(|| TxError::MalformedSpec {
                field: field.clone(),
                value: field_spec.to_string(),
                expected: "key=value, with the pairs separated by commas",
            })?;
        let value = value.trim();

        let key = match key.trim().to_ascii_lowercase().as_str() {
            "txid" | "prev_txid" | "prevtxid" => "txid",
            "vout" | "index" | "output_index" => "vout",
            "script_sig" | "scriptsig" | "sig" => "script_sig",
            "sequence" | "seq" => "sequence",
            "witness" | "wit" => "witness",
            other => {
                return Err(TxError::UnknownKey {
                    field: field.clone(),
                    key: other.to_string(),
                    valid: INPUT_KEYS,
                });
            }
        };

        if seen.contains(&key) {
            return Err(TxError::DuplicateKey {
                field: field.clone(),
                key: key.to_string(),
            });
        }
        seen.push(key);

        match key {
            "txid" => txid = Some(read_txid(&field, value, order)?),
            "vout" => vout = Some(parse_u32(&format!("{field} vout"), value)?),
            "script_sig" => script_sig = hex_to_bytes(&format!("{field} script_sig"), value)?,
            "sequence" => sequence = parse_u32(&format!("{field} sequence"), value)?,
            "witness" => {
                witness = parse_witness_items(&format!("{field} witness"), value)?;
                witness_given = true;
            }
            _ => unreachable!("keys are normalised above"),
        }
    }

    let Some(prev_txid) = txid else {
        return Err(TxError::MissingKey { field, key: "txid" });
    };
    let Some(vout) = vout else {
        return Err(TxError::MissingKey { field, key: "vout" });
    };

    Ok((
        TxInput {
            prev_txid,
            vout,
            script_sig,
            sequence,
            witness,
        },
        witness_given,
    ))
}

/// Read one `--output`.
fn parse_output(position: usize, spec: &str) -> Result<TxOutput> {
    let field = format!("--output #{}", position + 1);

    if !spec.contains('=') {
        // Shorthand: amount, then the script it pays to.
        let (amount, script) = spec.split_once(':').ok_or_else(|| TxError::MalformedSpec {
            field: field.clone(),
            value: spec.to_string(),
            expected: "satoshis:script_pubkey, or key=value pairs such as amount=...,script_pubkey=...",
        })?;
        return Ok(TxOutput {
            value: parse_amount(&format!("{field} amount"), amount.trim())?,
            script_pubkey: hex_to_bytes(&format!("{field} script_pubkey"), script.trim())?,
        });
    }

    let mut value: Option<u64> = None;
    let mut script_pubkey: Option<Vec<u8>> = None;
    let mut seen: Vec<&str> = Vec::new();

    for pair in spec.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        let (key, raw) = pair.split_once('=').ok_or_else(|| TxError::MalformedSpec {
            field: field.clone(),
            value: pair.to_string(),
            expected: "key=value, with the pairs separated by commas",
        })?;
        let raw = raw.trim();

        let key = match key.trim().to_ascii_lowercase().as_str() {
            "amount" | "value" | "sats" | "satoshis" => "amount",
            "script_pubkey" | "scriptpubkey" | "spk" | "script" => "script_pubkey",
            other => {
                return Err(TxError::UnknownKey {
                    field: field.clone(),
                    key: other.to_string(),
                    valid: OUTPUT_KEYS,
                });
            }
        };

        if seen.contains(&key) {
            return Err(TxError::DuplicateKey {
                field: field.clone(),
                key: key.to_string(),
            });
        }
        seen.push(key);

        match key {
            "amount" => value = Some(parse_amount(&format!("{field} amount"), raw)?),
            "script_pubkey" => {
                script_pubkey = Some(hex_to_bytes(&format!("{field} script_pubkey"), raw)?)
            }
            _ => unreachable!("keys are normalised above"),
        }
    }

    let Some(value) = value else {
        return Err(TxError::MissingKey {
            field,
            key: "amount",
        });
    };
    let Some(script_pubkey) = script_pubkey else {
        return Err(TxError::MissingKey {
            field,
            key: "script_pubkey",
        });
    };

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

/// A witness stack: items separated by `|`. Nothing at all means an empty
/// stack, which is legal. An input that needs no witness still has one.
fn parse_witness_items(field: &str, value: &str) -> Result<Vec<Vec<u8>>> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(Vec::new());
    }

    value
        .split('|')
        .enumerate()
        .map(|(position, item)| {
            hex_to_bytes(&format!("{field} item #{}", position + 1), item.trim())
        })
        .collect()
}

/// Decode the previous txid, flipping it into internal order unless the caller
/// says it is already there.
fn read_txid(field: &str, value: &str, order: TxidOrder) -> Result<Vec<u8>> {
    let mut bytes = hex_to_txid(&format!("{field} txid"), value)?;
    if order == TxidOrder::Display {
        // Explorers print a txid reversed; the transaction stores it as it was
        // hashed. Flipping here keeps the serialiser free of the distinction.
        bytes.reverse();
    }
    Ok(bytes)
}

/// Read a whole number written in decimal or as `0x` hex, with optional `_`
/// separators. The width check is left to the callers below.
fn parse_integer(field: &str, value: &str) -> Result<i128> {
    let cleaned = value.replace('_', "");
    let not_a_number = || TxError::NotANumber {
        field: field.to_string(),
        value: value.to_string(),
    };

    let (negative, digits) = match cleaned.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, cleaned.strip_prefix('+').unwrap_or(cleaned.as_str())),
    };
    let (radix, digits) = match digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        Some(rest) => (16, rest),
        None => (10, digits),
    };

    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return Err(not_a_number());
    }

    // Only a number too large for i128 can fail now, and that is a range
    // problem rather than a spelling one.
    let magnitude = i128::from_str_radix(digits, radix).map_err(|_| TxError::OutOfRange {
        field: field.to_string(),
        value: value.to_string(),
        limit: "a 128 bit number".to_string(),
    })?;

    Ok(if negative { -magnitude } else { magnitude })
}

fn parse_u32(field: &str, value: &str) -> Result<u32> {
    let number = parse_integer(field, value)?;
    u32::try_from(number).map_err(|_| TxError::OutOfRange {
        field: field.to_string(),
        value: value.to_string(),
        limit: "0 to 4294967295 (0xffffffff)".to_string(),
    })
}

fn parse_i32(field: &str, value: &str) -> Result<i32> {
    let number = parse_integer(field, value)?;
    i32::try_from(number).map_err(|_| TxError::OutOfRange {
        field: field.to_string(),
        value: value.to_string(),
        limit: "-2147483648 to 2147483647".to_string(),
    })
}

fn parse_index(field: &str, value: &str) -> Result<usize> {
    let number = parse_integer(field, value)?;
    usize::try_from(number).map_err(|_| TxError::OutOfRange {
        field: field.to_string(),
        value: value.to_string(),
        limit: "0 or greater".to_string(),
    })
}

fn parse_amount(field: &str, value: &str) -> Result<u64> {
    let number = parse_integer(field, value)?;
    if !(0..=MAX_MONEY).contains(&number) {
        return Err(TxError::OutOfRange {
            field: field.to_string(),
            value: value.to_string(),
            limit: "0 to 2100000000000000 satoshis, the whole supply".to_string(),
        });
    }
    Ok(number as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::bytes_to_hex;

    const TXID: &str = "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821";

    fn input(spec: &str) -> TxInput {
        parse_input(0, spec, TxidOrder::Internal).unwrap().0
    }

    #[test]
    fn shorthand_and_keys_describe_the_same_input() {
        let short = input(&format!("{TXID}:1"));
        let keyed = input(&format!("txid={TXID},vout=1"));
        assert_eq!(short, keyed);
        // Defaults: nothing to unlock with, and a final sequence.
        assert_eq!(short.script_sig, Vec::<u8>::new());
        assert_eq!(short.sequence, 0xffff_ffff);
        assert_eq!(short.witness, Vec::<Vec<u8>>::new());
    }

    #[test]
    fn display_order_txids_are_reversed_on_the_way_in() {
        let internal = parse_input(0, &format!("{TXID}:0"), TxidOrder::Internal)
            .unwrap()
            .0;
        let display = parse_input(0, &format!("{TXID}:0"), TxidOrder::Display)
            .unwrap()
            .0;

        assert_eq!(bytes_to_hex(&internal.prev_txid), TXID);
        let mut reversed = display.prev_txid.clone();
        reversed.reverse();
        assert_eq!(reversed, internal.prev_txid);
    }

    #[test]
    fn the_shorthand_may_carry_extra_fields() {
        let mixed = input(&format!("{TXID}:1,sequence=0xfffffffd,script_sig=aabb"));
        let keyed = input(&format!(
            "txid={TXID},vout=1,sequence=0xfffffffd,script_sig=aabb"
        ));
        assert_eq!(mixed, keyed);
        assert_eq!(mixed.sequence, 0xffff_fffd);
        assert_eq!(mixed.script_sig, vec![0xaa, 0xbb]);

        // The outpoint fills in txid and vout, so repeating either is still a
        // duplicate rather than a silent override.
        assert!(matches!(
            parse_input(0, &format!("{TXID}:1,vout=2"), TxidOrder::Display),
            Err(TxError::DuplicateKey { ref key, .. }) if key == "vout"
        ));
    }

    #[test]
    fn numbers_may_be_decimal_or_hex() {
        assert_eq!(input(&format!("txid={TXID},vout=0x02")).vout, 2);
        assert_eq!(
            input(&format!("txid={TXID},vout=0,sequence=0xfffffffd")).sequence,
            0xffff_fffd
        );
        assert_eq!(
            input(&format!("txid={TXID},vout=1_000_000")).vout,
            1_000_000
        );
        assert_eq!(
            parse_amount("f", "2_100_000_000_000_000").unwrap(),
            MAX_MONEY as u64
        );
    }

    #[test]
    fn out_of_range_numbers_are_rejected() {
        assert!(matches!(
            parse_u32("f", "4294967296"),
            Err(TxError::OutOfRange { .. })
        ));
        assert!(matches!(
            parse_u32("f", "-1"),
            Err(TxError::OutOfRange { .. })
        ));
        assert!(matches!(
            parse_u32("f", "12ab"),
            Err(TxError::NotANumber { .. })
        ));
        assert!(matches!(
            parse_u32("f", ""),
            Err(TxError::NotANumber { .. })
        ));
        // One satoshi past the whole supply.
        assert!(matches!(
            parse_amount("f", "2100000000000001"),
            Err(TxError::OutOfRange { .. })
        ));
        assert!(parse_amount("f", "2100000000000000").is_ok());
    }

    #[test]
    fn malformed_specs_name_the_argument_at_fault() {
        assert!(matches!(
            parse_input(1, "not-an-outpoint", TxidOrder::Display),
            Err(TxError::MalformedSpec { ref field, .. }) if field == "--input #2"
        ));
        assert!(matches!(
            parse_input(0, &format!("txid={TXID},vout=0,fee=100"), TxidOrder::Display),
            Err(TxError::UnknownKey { ref key, .. }) if key == "fee"
        ));
        assert!(matches!(
            parse_input(0, &format!("txid={TXID},vout=0,vout=1"), TxidOrder::Display),
            Err(TxError::DuplicateKey { .. })
        ));
        assert!(matches!(
            parse_input(0, "vout=0", TxidOrder::Display),
            Err(TxError::MissingKey { key: "txid", .. })
        ));
        assert!(matches!(
            parse_output(0, "amount=100"),
            Err(TxError::MissingKey {
                key: "script_pubkey",
                ..
            })
        ));
    }

    #[test]
    fn a_witness_stack_splits_on_pipes() {
        assert_eq!(parse_witness_items("w", "").unwrap(), Vec::<Vec<u8>>::new());
        assert_eq!(
            parse_witness_items("w", "aabb|cc").unwrap(),
            vec![vec![0xaa, 0xbb], vec![0xcc]]
        );
        // A leading empty item is how a P2WSH multisig stack starts.
        assert_eq!(
            parse_witness_items("w", "|aabb").unwrap(),
            vec![Vec::new(), vec![0xaa, 0xbb]]
        );
        assert!(matches!(
            parse_witness_items("w", "aabb|zz"),
            Err(TxError::InvalidHexChar { character: 'z', .. })
        ));
    }
}
