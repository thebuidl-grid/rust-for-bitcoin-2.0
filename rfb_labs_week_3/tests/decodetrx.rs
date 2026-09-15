//! Legacy and SegWit decoding tests for `decodetrx`.
//!
//! Both fixtures are real transactions whose expected values were taken from a
//! block explorer, not from this decoder — so the txid assertions are genuine
//! checks, not snapshots of our own output.

use decodetrx::decode_transaction;
use serde_json::Value;

/// Block 170, the Satoshi -> Hal Finney transaction. Pre-SegWit, so this
/// exercises the legacy path: no marker, no flag, no witness.
/// https://mempool.space/tx/f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16
const LEGACY_TX: &str = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";
const LEGACY_TXID: &str = "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16";

/// A testnet P2WPKH spend. Marker, flag, and a two-item witness stack.
/// https://blockstream.info/testnet/tx/be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b
const SEGWIT_TX: &str = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";
const SEGWIT_TXID: &str = "be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b";

fn decode(hex: &str) -> Value {
    serde_json::from_str(&decode_transaction(hex.to_string()).expect("decode should succeed"))
        .expect("decoder should emit valid JSON")
}

#[test]
fn legacy_transaction_id_matches_explorer() {
    assert_eq!(decode(LEGACY_TX)["transaction_id"], LEGACY_TXID);
}

#[test]
fn legacy_fields_are_parsed() {
    let tx = decode(LEGACY_TX);

    assert_eq!(tx["version"], 1);
    assert_eq!(tx["lock_time"], 0);
    assert_eq!(tx["inputs"].as_array().unwrap().len(), 1);
    assert_eq!(tx["outputs"].as_array().unwrap().len(), 2);

    // The previous txid is stored reversed on the wire; we display it the way
    // an explorer does.
    assert_eq!(
        tx["inputs"][0]["txid"],
        "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9"
    );
    assert_eq!(tx["inputs"][0]["output_index"], 0);
    assert_eq!(tx["inputs"][0]["sequence"], 0xffff_ffffu32);

    // 10 BTC to Hal, 40 BTC back as change.
    assert_eq!(tx["outputs"][0]["amount"], 10.0);
    assert_eq!(tx["outputs"][1]["amount"], 40.0);
}

#[test]
fn legacy_inputs_carry_no_witness() {
    // `witness` is skipped when empty, so a legacy input should not have the key.
    assert!(decode(LEGACY_TX)["inputs"][0].get("witness").is_none());
}

#[test]
fn segwit_transaction_id_strips_witness_data() {
    // The txid is the hash of the *legacy* serialization. If the marker, flag,
    // and witness were left in, this would produce the wtxid instead.
    assert_eq!(decode(SEGWIT_TX)["transaction_id"], SEGWIT_TXID);
}

#[test]
fn segwit_fields_are_parsed() {
    let tx = decode(SEGWIT_TX);

    assert_eq!(tx["version"], 2);
    assert_eq!(tx["lock_time"], 0);
    assert_eq!(tx["inputs"].as_array().unwrap().len(), 1);
    assert_eq!(tx["outputs"].as_array().unwrap().len(), 2);

    assert_eq!(
        tx["inputs"][0]["txid"],
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796"
    );
    assert_eq!(tx["inputs"][0]["output_index"], 1);

    // A native P2WPKH input has an empty scriptSig — the signature and pubkey
    // live in the witness instead.
    assert_eq!(tx["inputs"][0]["script_sig"], "");

    let witness = tx["inputs"][0]["witness"].as_array().unwrap();
    assert_eq!(witness.len(), 2, "P2WPKH witness is <signature> <pubkey>");
    assert_eq!(
        witness[1],
        "020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f1"
    );

    // 100 sats and 4_462_282 sats, per the explorer.
    assert_eq!(tx["outputs"][0]["amount"], 0.000001);
    assert_eq!(tx["outputs"][1]["amount"], 0.04462282);
}

#[test]
fn small_amounts_avoid_scientific_notation() {
    // 100 sats must render as 0.00000100, not 1e-6.
    let json = decode_transaction(SEGWIT_TX.to_string()).unwrap();
    assert!(json.contains("0.00000100"), "got: {}", json);
    assert!(!json.contains("1e-6"));
}

#[test]
fn read_version_reads_the_leading_four_bytes() {
    assert_eq!(decodetrx::read_version(LEGACY_TX), 1);
    assert_eq!(decodetrx::read_version(SEGWIT_TX), 2);
}

#[test]
fn read_version_rejects_bad_input() {
    assert_eq!(decodetrx::read_version("not hex"), 0);
    assert_eq!(decodetrx::read_version("0100"), 0, "fewer than 4 bytes");
}

#[test]
fn odd_length_hex_is_rejected() {
    assert!(decode_transaction("010000000".to_string()).is_err());
}

#[test]
fn truncated_transaction_is_rejected() {
    // Cut the transaction in half: the parser must run out of bytes and fail
    // rather than panic.
    let truncated = &LEGACY_TX[..LEGACY_TX.len() / 2];
    assert!(decode_transaction(truncated.to_string()).is_err());
}

#[test]
fn trailing_bytes_are_rejected() {
    let with_junk = format!("{}deadbeef", LEGACY_TX);
    let error = decode_transaction(with_junk).unwrap_err().to_string();
    assert!(error.contains("trailing"), "got: {}", error);
}

#[test]
fn bad_segwit_flag_is_rejected() {
    // Layout: version (chars 0..8), marker (8..10), flag (10..12), input count.
    // Leave the 0x00 marker alone so the SegWit branch is taken, then corrupt
    // only the flag byte.
    let mut broken = SEGWIT_TX.to_string();
    broken.replace_range(10..12, "02");

    let error = decode_transaction(broken).unwrap_err().to_string();
    assert!(error.contains("flag"), "got: {}", error);
}

#[test]
fn oversized_script_length_is_rejected_without_panicking() {
    // Claim a 0xffffffff-byte scriptSig. The guard in read_script_size must
    // reject this instead of trying to allocate 4 GB.
    let malicious = "0100000001\
                     c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704\
                     00000000\
                     feffffffff";
    assert!(decode_transaction(malicious.to_string()).is_err());
}
