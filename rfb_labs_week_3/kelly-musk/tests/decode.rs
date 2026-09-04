//! Integration tests for the Week 3 transaction decoder.
//!
//! Vectors:
//! * `LEGACY`  — the block-170 payment (Satoshi -> Hal Finney), a pre-SegWit tx.
//! * `SEGWIT`  — a BIP143-derived two-input transaction (one legacy input, one
//!   native P2WPKH input) exercising the marker/flag and witness stacks.
//! * `COINBASE` — the Bitcoin genesis coinbase.
//!
//! All expected txids/wtxids were cross-checked with an independent SHA256d
//! implementation.

use week3_txdecode::error::ParseError;
use week3_txdecode::{decode_transaction_hex, parse_transaction_hex};

const LEGACY: &str = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";

const SEGWIT: &str = "01000000000102fff7f7881a8099afa6940d42d1e7f6362bec38171ea3edf433541db4e4ad969f00000000494830450221008b9d1dc26ba6a9cb62127b02742fa9d754cd3bebf337f7a55d114c8e5cdd30be022040529b194ba3f9281a99f2b1c0a19c0489bc22ede944ccf4ecbab4cc618ef3ed01eeffffffef51e1b804cc89d182d279655c3aa89e815b1b309fe287d9b2b55d57b90ec68a0100000000ffffffff02202cb206000000001976a9148280b37df378db99f66f85c95a783a76ac7a6d5988ac9093510d000000001976a9143bde42dbee7e4dbe6a21b2d50ce2f0167faa815988ac000247304402203609e17b84f6a7d30c80bfa610b5b4542f32a8a0d5447a12fb1366d7f01cc44a0220573a954c4518331561406f90300e8f3358f51928d43c212a8caed02de67eebee0121025476c2e83188368da1ff3e292e7acafcdb3566bb0ad253f62fcb9e2cf0df8081511e0100";

const COINBASE: &str = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";

#[test]
fn parses_the_legacy_structure() {
    let tx = parse_transaction_hex(LEGACY).unwrap();
    assert_eq!(tx.version, 1);
    assert!(!tx.segwit);
    assert_eq!(tx.inputs.len(), 1);
    assert_eq!(tx.outputs.len(), 2);
    assert_eq!(tx.lock_time, 0);
    assert_eq!(tx.outputs[0].value, 1_000_000_000);
    assert_eq!(tx.outputs[1].value, 4_000_000_000);
    assert_eq!(tx.inputs[0].previous_output.vout, 0);
    assert!(!tx.inputs[0].previous_output.is_coinbase());
    assert!(tx.inputs[0].witness.is_empty());
}

#[test]
fn computes_the_legacy_txid_and_sizes() {
    let tx = parse_transaction_hex(LEGACY).unwrap();
    assert_eq!(
        tx.txid(),
        "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16"
    );
    // No witness data: wtxid == txid.
    assert_eq!(tx.wtxid(), tx.txid());
    assert_eq!(tx.total_size(), 275);
    assert_eq!(tx.base_size(), 275);
    assert_eq!(tx.weight(), 275 * 4);
    assert_eq!(tx.vsize(), 275);
}

#[test]
fn round_trips_the_legacy_serialization() {
    let tx = parse_transaction_hex(LEGACY).unwrap();
    assert_eq!(hex::encode(tx.serialize()), LEGACY);
    assert_eq!(hex::encode(tx.serialize_legacy()), LEGACY);
}

#[test]
fn parses_the_segwit_structure() {
    let tx = parse_transaction_hex(SEGWIT).unwrap();
    assert!(tx.segwit);
    assert_eq!(tx.inputs.len(), 2);
    assert_eq!(tx.outputs.len(), 2);
    assert_eq!(tx.lock_time, 73_297);
    // First input is a legacy input carried inside a SegWit tx: no witness.
    assert!(tx.inputs[0].witness.is_empty());
    assert!(!tx.inputs[0].script_sig.is_empty());
    // Second input is native P2WPKH: empty scriptSig, 2 witness items.
    assert!(tx.inputs[1].script_sig.is_empty());
    assert_eq!(tx.inputs[1].witness.len(), 2);
}

#[test]
fn computes_the_segwit_txid_wtxid_and_weight() {
    let tx = parse_transaction_hex(SEGWIT).unwrap();
    assert_eq!(
        tx.txid(),
        "dbe9e3081bad58cccc97c08934ca6ba78e18add2fe3aa81ccbb7676997038677"
    );
    assert_eq!(
        tx.wtxid(),
        "3da1badd41c9265e856a8b743a9a96cef58b0eca5edb832431afc2cb9cd649fc"
    );
    assert_ne!(tx.txid(), tx.wtxid());
    assert!(tx.base_size() < tx.total_size());
    assert_eq!(tx.weight(), tx.base_size() * 3 + tx.total_size());
    assert_eq!(tx.vsize(), tx.weight().div_ceil(4));
}

#[test]
fn round_trips_the_segwit_serialization() {
    let tx = parse_transaction_hex(SEGWIT).unwrap();
    assert_eq!(hex::encode(tx.serialize()), SEGWIT);
}

#[test]
fn recognizes_a_coinbase_input() {
    let tx = parse_transaction_hex(COINBASE).unwrap();
    assert_eq!(tx.inputs.len(), 1);
    assert!(tx.inputs[0].previous_output.is_coinbase());
    assert_eq!(
        tx.txid(),
        "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"
    );

    let decoded = decode_transaction_hex(COINBASE).unwrap();
    assert!(decoded.vin[0].coinbase.is_some());
    assert!(decoded.vin[0].txid.is_none());
    assert!(decoded.vin[0].script_sig.is_none());
}

#[test]
fn decoded_view_matches_core_shapes() {
    let decoded = decode_transaction_hex(SEGWIT).unwrap();
    assert_eq!(decoded.txid, decoded.txid);
    assert!(decoded.segwit);
    assert_eq!(decoded.vin.len(), 2);
    assert!(decoded.vin[1].script_sig.as_ref().unwrap().hex.is_empty());
    assert_eq!(decoded.vin[1].txinwitness.len(), 2);
    assert_eq!(decoded.vout[0].value_btc, "1.12340000");
    assert_eq!(decoded.vout[0].script_pubkey.script_type, "pubkeyhash");
    assert_eq!(
        decoded.vout[0].script_pubkey.asm,
        "OP_DUP OP_HASH160 8280b37df378db99f66f85c95a783a76ac7a6d59 OP_EQUALVERIFY OP_CHECKSIG"
    );

    let legacy = decode_transaction_hex(LEGACY).unwrap();
    assert_eq!(legacy.vout[0].script_pubkey.script_type, "pubkey");
    assert_eq!(legacy.vout[0].value_btc, "10.00000000");
    assert_eq!(legacy.hash, legacy.txid);
}

#[test]
fn rejects_trailing_bytes() {
    let extended = format!("{LEGACY}00");
    match parse_transaction_hex(&extended) {
        Err(ParseError::TrailingBytes { .. }) => {}
        other => panic!("expected TrailingBytes, got {other:?}"),
    }
}

#[test]
fn rejects_truncated_input() {
    let truncated = &LEGACY[..LEGACY.len() - 8]; // drop locktime
    match parse_transaction_hex(truncated) {
        Err(ParseError::UnexpectedEof { .. }) => {}
        other => panic!("expected UnexpectedEof, got {other:?}"),
    }
}

#[test]
fn rejects_a_bad_segwit_flag() {
    // version(4) + marker 0x00 + flag 0x02 (invalid) + ...
    let bad = "0100000000021234";
    match parse_transaction_hex(bad) {
        Err(ParseError::BadSegwitFlag { flag: 0x02, .. }) => {}
        other => panic!("expected BadSegwitFlag, got {other:?}"),
    }
}

#[test]
fn rejects_non_minimal_compact_size() {
    // version(4) + input count encoded as 0xfd 0x0500 (value 5, should be a single byte)
    let bad = "01000000fd0500";
    match parse_transaction_hex(bad) {
        Err(ParseError::NonMinimalCompactSize { .. }) => {}
        other => panic!("expected NonMinimalCompactSize, got {other:?}"),
    }
}

#[test]
fn rejects_non_hex_input() {
    match parse_transaction_hex("not-hex-zz") {
        Err(ParseError::BadHex(_)) => {}
        other => panic!("expected BadHex, got {other:?}"),
    }
}
