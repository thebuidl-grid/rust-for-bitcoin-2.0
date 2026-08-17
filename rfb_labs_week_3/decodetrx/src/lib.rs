use sha2::{Digest, Sha256};
use std::io::{Error, ErrorKind};
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

/// Quick-peek at the version field straight out of the hex *string*, without
/// touching a byte cursor at all. Bitcoin's version field is always the first
/// 4 bytes = the first 8 hex characters, encoded little-endian.
///
/// This exists side-by-side with [`read_version_byte`] on purpose: it is the
/// same field read two different ways (string slicing vs. byte-cursor
/// consumption) so the two techniques can be compared directly.
#[allow(unused_variables)]
fn read_version(transaction_hex: &str) -> u32 {
    let version_hex = &transaction_hex[0..8];
    let bytes = hex::decode(version_hex).expect("version field must be valid hex");
    u32::from_le_bytes(
        bytes
            .try_into()
            .expect("version field must be exactly 4 bytes"),
    )
}

/// Reads the next 8 bytes off the front of `transaction_bytes` as a
/// little-endian `u64`, advancing the slice past them.
///
/// Unlike its siblings below, this one has no `Result` in its signature: it
/// trusts the caller to only call it when at least 8 bytes remain (exactly
/// the contract `read_amount` relies on). Reaching for `Result` everywhere
/// is usually the right call, but this mirrors the assignment's original
/// function signature and doubles as an example of an "infallible by
/// contract" helper versus the `Result`-returning ones nearby.
fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let (value_bytes, rest) = transaction_bytes.split_at(8);
    *transaction_bytes = rest;
    u64::from_le_bytes(value_bytes.try_into().unwrap())
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    if transaction_bytes.len() < 8 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "expected 8 bytes for an output amount",
        ));
    }
    Ok(Amount::from_sat(read_u64(transaction_bytes)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    if bytes_slice.len() < 4 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "expected 4 bytes for a u32 field",
        ));
    }
    let (value_bytes, rest) = bytes_slice.split_at(4);
    *bytes_slice = rest;
    Ok(u32::from_le_bytes(value_bytes.try_into().unwrap()))
}

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

/// Reads a Bitcoin CompactSize ("VarInt"): a length-prefixed integer used
/// everywhere a count or a byte-length needs to be encoded (input count,
/// output count, script lengths, witness item counts, witness item lengths).
fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.is_empty() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "expected at least 1 byte for a CompactSize prefix",
        ));
    }
    let (prefix, rest) = transaction_bytes.split_at(1);
    *transaction_bytes = rest;

    match prefix[0] {
        0xfd => {
            if transaction_bytes.len() < 2 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "expected 2 bytes after a 0xfd CompactSize prefix",
                ));
            }
            let (b, rest) = transaction_bytes.split_at(2);
            *transaction_bytes = rest;
            Ok(u16::from_le_bytes(b.try_into().unwrap()) as u64)
        }
        0xfe => {
            if transaction_bytes.len() < 4 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "expected 4 bytes after a 0xfe CompactSize prefix",
                ));
            }
            let (b, rest) = transaction_bytes.split_at(4);
            *transaction_bytes = rest;
            Ok(u32::from_le_bytes(b.try_into().unwrap()) as u64)
        }
        0xff => {
            if transaction_bytes.len() < 8 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "expected 8 bytes after a 0xff CompactSize prefix",
                ));
            }
            Ok(read_u64(transaction_bytes))
        }
        small => Ok(small as u64),
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    if transaction_bytes.len() < 32 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "expected 32 bytes for a txid",
        ));
    }
    let (txid_bytes, rest) = transaction_bytes.split_at(32);
    *transaction_bytes = rest;
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(txid_bytes);
    Ok(Txid::from_bytes(buffer))
}

/// Reads a CompactSize length prefix followed by that many bytes, returning
/// them hex-encoded. Both scriptSig/scriptPubKey and witness stack items
/// share this exact shape (CompactSize length + raw bytes), so this one
/// function is reused for all of them in `decode_transaction`.
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let length = read_compact_size(transaction_bytes)? as usize;
    if transaction_bytes.len() < length {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "script/witness item length prefix exceeds remaining bytes",
        ));
    }
    let (script_bytes, rest) = transaction_bytes.split_at(length);
    *transaction_bytes = rest;
    Ok(hex::encode(script_bytes))
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

/// Bitcoin's txid/wtxid are double-SHA256: `SHA256(SHA256(data))`. The
/// digest comes out of `sha2` in "internal" byte order; `Txid` keeps it in
/// that order and reverses only when displaying/serializing (see
/// `Txid::to_hex`), matching how Bitcoin Core itself separates storage from
/// display.
fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_pass = Sha256::digest(row_transaction_bytes);
    let second_pass = Sha256::digest(first_pass);
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(&second_pass);
    Ok(Txid::from_bytes(buffer))
}

/// Decodes a raw Bitcoin transaction given as a hex string and returns a
/// pretty-printed JSON description of it, in the same spirit as Bitcoin
/// Core's `decoderawtransaction` RPC.
pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let clean_hex = transaction_hex.trim();
    let raw = hex::decode(clean_hex)?;

    let mut cursor: &[u8] = &raw[..];

    let version = read_version_byte(&mut cursor)?;
    debug_assert_eq!(
        version,
        read_version(clean_hex),
        "string-slice and byte-cursor version reads must agree"
    );

    // A SegWit transaction inserts a 2-byte marker (0x00) + flag (0x01)
    // right after the version, before the input count. A marker of 0x00
    // can never collide with a real input count, because a transaction with
    // zero inputs is never valid - that ambiguity is exactly how old nodes
    // and new nodes can tell legacy and SegWit transactions apart.
    let is_segwit = cursor.len() >= 2 && cursor[0] == 0x00 && cursor[1] == 0x01;
    if is_segwit {
        cursor = &cursor[2..];
    }

    // Remember where the "legacy body" (input count..outputs) starts and
    // ends in the original buffer, so we can reassemble the exact bytes a
    // pre-SegWit node would have hashed for the txid, without re-encoding
    // anything by hand.
    let body_start = raw.len() - cursor.len();

    let input_count = read_compact_size(&mut cursor)?;
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        let txid = read_txid(&mut cursor)?;
        let output_index = read_u32(&mut cursor)?;
        let script_sig = hex::decode(read_script_size(&mut cursor)?)?;
        let sequence = read_u32(&mut cursor)?;
        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
            witness: Vec::new(),
        });
    }

    let output_count = read_compact_size(&mut cursor)?;
    let mut outputs = Vec::with_capacity(output_count as usize);
    for _ in 0..output_count {
        let amount = read_amount(&mut cursor)?;
        let script_pubkey = hex::decode(read_script_size(&mut cursor)?)?;
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    let body_end = raw.len() - cursor.len();

    if is_segwit {
        // Every input has its own witness stack, listed in the same order
        // as the inputs themselves.
        for input in inputs.iter_mut() {
            let item_count = read_compact_size(&mut cursor)?;
            for _ in 0..item_count {
                input
                    .witness
                    .push(hex::decode(read_script_size(&mut cursor)?)?);
            }
        }
    }

    let lock_time = read_u32(&mut cursor)?;

    if !cursor.is_empty() {
        return Err(format!(
            "{} unexpected trailing byte(s) after locktime",
            cursor.len()
        )
        .into());
    }

    // txid = hash of the legacy serialization only: version + inputs +
    // outputs + locktime, with the marker/flag/witness stripped out. That
    // is precisely `body_start..body_end` sandwiched between the version
    // and the locktime, so we can slice it straight out of `raw` instead of
    // re-serializing the parsed fields by hand.
    let mut legacy_bytes = Vec::with_capacity(4 + (body_end - body_start) + 4);
    legacy_bytes.extend_from_slice(&raw[0..4]);
    legacy_bytes.extend_from_slice(&raw[body_start..body_end]);
    legacy_bytes.extend_from_slice(&lock_time.to_le_bytes());
    let transaction_id = hash_row_transaction(&legacy_bytes)?;

    // wtxid = hash of the *entire* wire serialization, marker/flag/witness
    // included. Legacy transactions have no witness data to hash
    // separately, so they have no distinct wtxid.
    let wtxid = if is_segwit {
        Some(hash_row_transaction(&raw)?)
    } else {
        None
    };

    let transaction = Transaction {
        transaction_id,
        wtxid,
        version,
        is_segwit,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Block 170: the first-ever peer-to-peer Bitcoin transaction (Satoshi -> Hal Finney).
    // https://mempool.space/tx/f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16
    const LEGACY_TX_HEX: &str = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";
    const LEGACY_TXID: &str = "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16";

    // A real, confirmed mainnet SegWit (P2WPKH -> P2SH) spend, block 962834.
    // https://mempool.space/tx/95c3972fc972b409e214cef97d7564ee30a3cc48e016a9c0b45d155ff4979e0e
    const SEGWIT_TX_HEX: &str = "01000000000101c211518bfa3543cc614d679d63496bea14b1ffdda4b37fc8b6bc450a83c2d62c0100000000ffffffff01943d00000000000017a914633dc24b669a63d375fb7c13fb907828a420a55e8702483045022100c2b5eb29fd74b306decc2094b9db88f39eb2cae0553dcf5490d0db5bdea72a74022052c2221bd7c9450d44f0da16ef8d79bef300daa089dbeebb45d21f8e7e073a6b012103240dd00f0b13416e666877c34f2c658429a45a94d513bf2237f260891ca058cb00000000";
    const SEGWIT_TXID: &str = "95c3972fc972b409e214cef97d7564ee30a3cc48e016a9c0b45d155ff4979e0e";
    const SEGWIT_WTXID: &str = "45743ea5b54e4f730e4b08bb23e542f86ff515869f46b8a50a9ace81e090062e";

    #[test]
    fn decodes_legacy_transaction() {
        let json = decode_transaction(LEGACY_TX_HEX.to_string()).expect("valid legacy tx");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["transaction_id"], LEGACY_TXID);
        assert_eq!(value["version"], 1);
        assert_eq!(value["is_segwit"], false);
        assert!(value["wtxid"].is_null());
        assert_eq!(value["lock_time"], 0);

        assert_eq!(value["inputs"].as_array().unwrap().len(), 1);
        assert_eq!(value["inputs"][0]["output_index"], 0);
        assert_eq!(value["inputs"][0]["sequence"], 0xffffffffu32);
        assert!(value["inputs"][0]["witness"].as_array().unwrap().is_empty());

        assert_eq!(value["outputs"].as_array().unwrap().len(), 2);
        assert_eq!(value["outputs"][0]["amount"], 10.0);
        assert_eq!(value["outputs"][1]["amount"], 40.0);
    }

    #[test]
    fn decodes_segwit_transaction_and_separates_txid_from_wtxid() {
        let json = decode_transaction(SEGWIT_TX_HEX.to_string()).expect("valid segwit tx");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["transaction_id"], SEGWIT_TXID);
        assert_eq!(value["wtxid"], SEGWIT_WTXID);
        assert_eq!(value["is_segwit"], true);

        // The scriptSig is empty because the unlocking data lives in the
        // witness instead, which is exactly what makes this SegWit.
        assert_eq!(value["inputs"][0]["script_sig"], "");
        assert_eq!(value["inputs"][0]["witness"].as_array().unwrap().len(), 2);

        assert_eq!(value["outputs"].as_array().unwrap().len(), 1);
        assert_eq!(value["outputs"][0]["amount"], 0.00015764);
    }

    #[test]
    fn read_compact_size_handles_all_size_classes() {
        let mut one_byte: &[u8] = &[0x05];
        assert_eq!(read_compact_size(&mut one_byte).unwrap(), 5);

        let mut two_byte: &[u8] = &[0xfd, 0x00, 0x01]; // 256
        assert_eq!(read_compact_size(&mut two_byte).unwrap(), 256);

        let mut four_byte: &[u8] = &[0xfe, 0x00, 0x00, 0x01, 0x00]; // 65536
        assert_eq!(read_compact_size(&mut four_byte).unwrap(), 65536);

        let mut eight_byte: &[u8] = &[0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]; // 2^32
        assert_eq!(read_compact_size(&mut eight_byte).unwrap(), 1u64 << 32);
    }

    #[test]
    fn read_compact_size_reports_truncated_input() {
        let mut truncated: &[u8] = &[0xfd, 0x01]; // says "read 2 more bytes" but only 1 remains
        assert!(read_compact_size(&mut truncated).is_err());
    }

    #[test]
    fn decode_transaction_rejects_garbage_hex() {
        assert!(decode_transaction("not-hex".to_string()).is_err());
    }

    #[test]
    fn decode_transaction_rejects_truncated_transaction() {
        // Chop the legacy transaction hex off halfway through the first input.
        let truncated = &LEGACY_TX_HEX[0..40];
        assert!(decode_transaction(truncated.to_string()).is_err());
    }
}
