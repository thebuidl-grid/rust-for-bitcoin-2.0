//! What the program prints.
//!
//! The hex and the size are the point; `--verbose` adds a reading of the same
//! transaction field by field, so it is possible to check the values that went
//! in without decoding the bytes that came out.

use crate::hex::bytes_to_hex;
use crate::transaction::Transaction;

/// Render the finished transaction for the terminal.
pub fn render(trx: &Transaction, serialized: &[u8], verbose: bool, show_bytes: bool) -> String {
    let mut out = String::new();

    if verbose {
        out.push_str(&breakdown(trx));
        out.push('\n');
    }

    if show_bytes {
        out.push_str("Serialized transaction (bytes):\n");
        out.push_str(&format!("{:?}\n\n", serialized));
    }

    out.push_str("Serialized transaction (hex):\n");
    out.push_str(&bytes_to_hex(serialized));
    out.push_str("\n\n");
    out.push_str(&format!("Transaction size: {} bytes\n", serialized.len()));

    out
}

/// The field by field reading shown with `--verbose`.
fn breakdown(trx: &Transaction) -> String {
    let mut out = String::new();

    out.push_str("Transaction\n");
    out.push_str(&row("version", &trx.version.to_string()));
    out.push_str(&row(
        "format",
        if trx.segwit {
            "SegWit (BIP144 marker and flag, witness section)"
        } else {
            "legacy (no marker, flag or witness section)"
        },
    ));
    out.push_str(&row("locktime", &describe_locktime(trx.locktime)));

    out.push_str(&format!("\nInputs ({})\n", trx.inputs.len()));
    for (index, input) in trx.inputs.iter().enumerate() {
        // The transaction stores the txid in internal order; reverse it back
        // to the order it is quoted in everywhere else.
        let mut txid = input.prev_txid.clone();
        txid.reverse();

        out.push_str(&format!("  #{index}\n"));
        out.push_str(&row(
            "  outpoint",
            &format!("{}:{}", bytes_to_hex(&txid), input.vout),
        ));
        out.push_str(&row(
            "  script_sig",
            &if input.script_sig.is_empty() {
                "empty".to_string()
            } else {
                format!(
                    "{} bytes  {}",
                    input.script_sig.len(),
                    bytes_to_hex(&input.script_sig)
                )
            },
        ));
        out.push_str(&row(
            "  sequence",
            &format!(
                "{:#010x}  ({})",
                input.sequence,
                describe_sequence(input.sequence)
            ),
        ));

        if trx.segwit {
            out.push_str(&row(
                "  witness",
                &match input.witness.len() {
                    0 => "empty stack".to_string(),
                    1 => "1 item".to_string(),
                    n => format!("{n} items"),
                },
            ));
            for (item_index, item) in input.witness.iter().enumerate() {
                out.push_str(&format!(
                    "                   [{}] {:>3} bytes  {}\n",
                    item_index,
                    item.len(),
                    bytes_to_hex(item)
                ));
            }
        }
    }

    out.push_str(&format!("\nOutputs ({})\n", trx.outputs.len()));
    for (index, output) in trx.outputs.iter().enumerate() {
        out.push_str(&format!("  #{index}\n"));
        out.push_str(&row(
            "  amount",
            &format!(
                "{} sat  ({} BTC)",
                group_digits(output.value),
                to_btc(output.value)
            ),
        ));
        out.push_str(&row(
            "  script_pubkey",
            &format!(
                "{} bytes, {}",
                output.script_pubkey.len(),
                describe_script(&output.script_pubkey)
            ),
        ));
        out.push_str(&format!(
            "                   {}\n",
            bytes_to_hex(&output.script_pubkey)
        ));
    }

    out.push_str(&format!(
        "\n  total out        {} sat  ({} BTC)\n",
        group_digits(trx.total_output_value()),
        to_btc(trx.total_output_value())
    ));

    out.push_str("\nSize\n");
    out.push_str(&row("base", &format!("{} bytes", trx.base_size())));
    out.push_str(&row("total", &format!("{} bytes", trx.total_size())));
    out.push_str(&row("weight", &format!("{} WU", trx.weight())));
    out.push_str(&row("virtual size", &format!("{} vB", trx.vsize())));

    out
}

fn row(label: &str, value: &str) -> String {
    format!("  {:<17}{}\n", label, value)
}

/// Locktime is two things in one field, split at 500,000,000.
fn describe_locktime(locktime: u32) -> String {
    match locktime {
        0 => "0  (no locktime)".to_string(),
        n if n < 500_000_000 => format!("{n}  (block height)"),
        n => format!("{n}  (Unix time)"),
    }
}

/// What an input's sequence number says about locktime and replaceability.
fn describe_sequence(sequence: u32) -> &'static str {
    match sequence {
        0xffff_ffff => "final",
        0xffff_fffe => "locktime enabled",
        _ => "replaceable, BIP125",
    }
}

/// Name the standard output scripts by their shape. Anything unrecognised is
/// still serialised, it just does not get a label.
fn describe_script(script: &[u8]) -> &'static str {
    match script {
        [0x76, 0xa9, 0x14, .., 0x88, 0xac] if script.len() == 25 => "P2PKH",
        [0xa9, 0x14, .., 0x87] if script.len() == 23 => "P2SH",
        [0x00, 0x14, ..] if script.len() == 22 => "P2WPKH",
        [0x00, 0x20, ..] if script.len() == 34 => "P2WSH",
        [0x51, 0x20, ..] if script.len() == 34 => "P2TR",
        [0x6a, ..] => "OP_RETURN data",
        [.., 0xac] if script.len() == 35 || script.len() == 67 => "P2PK",
        [] => "empty script",
        _ => "non standard",
    }
}

/// Satoshis as BTC, exactly: eight decimal places, no floating point.
fn to_btc(satoshis: u64) -> String {
    format!("{}.{:08}", satoshis / 100_000_000, satoshis % 100_000_000)
}

/// Group a satoshi amount in threes so large numbers stay readable.
fn group_digits(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (position, digit) in digits.chars().enumerate() {
        if position > 0 && (digits.len() - position) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::hex_to_bytes;

    #[test]
    fn recognises_the_standard_scripts() {
        let script = |hex: &str| hex_to_bytes("f", hex).unwrap();
        assert_eq!(
            describe_script(&script("0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b")),
            "P2WPKH"
        );
        assert_eq!(
            describe_script(&script(
                "76a914a632c1fff47af29f8c81dc4c6e91eb49a116c12b88ac"
            )),
            "P2PKH"
        );
        assert_eq!(
            describe_script(&script("a914a632c1fff47af29f8c81dc4c6e91eb49a116c12b87")),
            "P2SH"
        );
        assert_eq!(describe_script(&script("6a0568656c6c6f")), "OP_RETURN data");
        assert_eq!(describe_script(&[]), "empty script");
        assert_eq!(describe_script(&script("51")), "non standard");
    }

    #[test]
    fn amounts_are_exact_and_readable() {
        assert_eq!(to_btc(69_886), "0.00069886");
        assert_eq!(to_btc(2_100_000_000_000_000), "21000000.00000000");
        assert_eq!(to_btc(0), "0.00000000");
        assert_eq!(group_digits(1), "1");
        assert_eq!(group_digits(1_000), "1,000");
        assert_eq!(group_digits(2_100_000_000_000_000), "2,100,000,000,000,000");
    }

    #[test]
    fn locktime_reads_as_a_height_or_a_time() {
        assert_eq!(describe_locktime(0), "0  (no locktime)");
        assert_eq!(describe_locktime(500_000), "500000  (block height)");
        assert_eq!(describe_locktime(1_700_000_000), "1700000000  (Unix time)");
    }
}
