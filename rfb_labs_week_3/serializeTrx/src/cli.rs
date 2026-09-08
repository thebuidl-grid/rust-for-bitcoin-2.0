use std::collections::HashMap;

use crate::error::{hex_to_bytes, TxArgError};
use crate::transaction::{TxInput, TxOutput};

/// Split a `key=value,key=value` argument into a field map, rejecting malformed pairs.
fn parse_fields(arg: &'static str, raw: &str) -> Result<HashMap<String, String>, TxArgError> {
    let mut fields = HashMap::new();
    for pair in raw.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| TxArgError::InvalidPair {
                arg,
                raw: pair.to_string(),
            })?;
        fields.insert(key.trim().to_string(), value.trim().to_string());
    }
    Ok(fields)
}

fn require_no_unknown_fields(
    arg: &'static str,
    fields: &HashMap<String, String>,
    allowed: &[&str],
) -> Result<(), TxArgError> {
    for key in fields.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(TxArgError::UnknownField {
                arg,
                field: key.clone(),
            });
        }
    }
    Ok(())
}

fn parse_hex_field(
    arg: &'static str,
    field: &'static str,
    raw: &str,
) -> Result<Vec<u8>, TxArgError> {
    hex_to_bytes(raw).map_err(|source| TxArgError::InvalidHex {
        arg,
        field,
        raw: raw.to_string(),
        source,
    })
}

fn parse_u32_field(arg: &'static str, field: &'static str, raw: &str) -> Result<u32, TxArgError> {
    raw.parse::<u32>().map_err(|e| TxArgError::InvalidInt {
        arg,
        field,
        raw: raw.to_string(),
        reason: e.to_string(),
    })
}

fn parse_u64_field(arg: &'static str, field: &'static str, raw: &str) -> Result<u64, TxArgError> {
    raw.parse::<u64>().map_err(|e| TxArgError::InvalidInt {
        arg,
        field,
        raw: raw.to_string(),
        reason: e.to_string(),
    })
}

/// Parse a `--input` argument, e.g.:
/// `txid=<hex>,vout=<u32>[,sequence=<u32>][,script_sig=<hex>][,witness=<hex>|<hex>|...]`
pub fn parse_input(raw: &str) -> Result<TxInput, String> {
    parse_input_inner(raw).map_err(|e| e.to_string())
}

fn parse_input_inner(raw: &str) -> Result<TxInput, TxArgError> {
    const ARG: &str = "--input";
    let fields = parse_fields(ARG, raw)?;
    require_no_unknown_fields(
        ARG,
        &fields,
        &["txid", "vout", "sequence", "script_sig", "witness"],
    )?;

    let txid_hex = fields.get("txid").ok_or(TxArgError::MissingField {
        arg: ARG,
        field: "txid",
    })?;
    let prev_txid = parse_hex_field(ARG, "txid", txid_hex)?;
    if prev_txid.len() != 32 {
        return Err(TxArgError::InvalidTxidLength {
            length: prev_txid.len(),
        });
    }

    let vout_raw = fields.get("vout").ok_or(TxArgError::MissingField {
        arg: ARG,
        field: "vout",
    })?;
    let vout = parse_u32_field(ARG, "vout", vout_raw)?;

    let sequence = match fields.get("sequence") {
        Some(raw) => parse_u32_field(ARG, "sequence", raw)?,
        None => 0xffff_ffff,
    };

    let script_sig = match fields.get("script_sig") {
        Some(raw) if !raw.is_empty() => parse_hex_field(ARG, "script_sig", raw)?,
        _ => Vec::new(),
    };

    let witness = match fields.get("witness") {
        Some(raw) if !raw.is_empty() => raw
            .split('|')
            .map(|item| parse_hex_field(ARG, "witness", item))
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness,
    })
}

/// Parse an `--output` argument, e.g. `value=<u64>,script_pubkey=<hex>`.
pub fn parse_output(raw: &str) -> Result<TxOutput, String> {
    parse_output_inner(raw).map_err(|e| e.to_string())
}

fn parse_output_inner(raw: &str) -> Result<TxOutput, TxArgError> {
    const ARG: &str = "--output";
    let fields = parse_fields(ARG, raw)?;
    require_no_unknown_fields(ARG, &fields, &["value", "script_pubkey"])?;

    let value_raw = fields.get("value").ok_or(TxArgError::MissingField {
        arg: ARG,
        field: "value",
    })?;
    let value = parse_u64_field(ARG, "value", value_raw)?;

    let script_pubkey_hex = fields
        .get("script_pubkey")
        .ok_or(TxArgError::MissingField {
            arg: ARG,
            field: "script_pubkey",
        })?;
    let script_pubkey = parse_hex_field(ARG, "script_pubkey", script_pubkey_hex)?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_input() {
        let input = parse_input(
            "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1,sequence=4294967295,script_sig=,witness=aa|bb",
        )
        .unwrap();
        assert_eq!(input.vout, 1);
        assert_eq!(input.sequence, 0xffff_ffff);
        assert!(input.script_sig.is_empty());
        assert_eq!(input.witness, vec![vec![0xaa], vec![0xbb]]);
    }

    #[test]
    fn defaults_sequence_when_absent() {
        let input = parse_input(
            "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=0",
        )
        .unwrap();
        assert_eq!(input.sequence, 0xffff_ffff);
        assert!(input.witness.is_empty());
    }

    #[test]
    fn rejects_odd_length_hex() {
        let err = parse_input("txid=abc,vout=0").unwrap_err();
        assert!(err.contains("odd number"));
    }

    #[test]
    fn rejects_non_hex_chars() {
        let err = parse_input("txid=zz,vout=0").unwrap_err();
        assert!(err.contains("invalid hex character"));
    }

    #[test]
    fn rejects_wrong_txid_length() {
        let err = parse_input("txid=aabb,vout=0").unwrap_err();
        assert!(err.contains("32 bytes"));
    }

    #[test]
    fn rejects_missing_field() {
        let err = parse_input("vout=0").unwrap_err();
        assert!(err.contains("missing required field 'txid'"));
    }

    #[test]
    fn rejects_unknown_field() {
        let err = parse_input(
            "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=0,foo=bar",
        )
        .unwrap_err();
        assert!(err.contains("unknown field 'foo'"));
    }

    #[test]
    fn rejects_malformed_pair() {
        let err = parse_input("txid").unwrap_err();
        assert!(err.contains("expected 'key=value'"));
    }

    #[test]
    fn parses_output() {
        let output =
            parse_output("value=69886,script_pubkey=0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b")
                .unwrap();
        assert_eq!(output.value, 69886);
        assert_eq!(output.script_pubkey.len(), 22);
    }

    #[test]
    fn rejects_invalid_integer() {
        let err = parse_output("value=notanumber,script_pubkey=aa").unwrap_err();
        assert!(err.contains("invalid integer value"));
    }
}
