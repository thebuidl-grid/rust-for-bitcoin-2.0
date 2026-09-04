//! Tests for the transaction serializer.
//!
//! The two headline tests rebuild real on-chain transactions from CLI-style
//! specs and assert the output matches the raw hex byte for byte. Those raw
//! transactions came from block explorers, so they are genuine checks rather
//! than snapshots of this serializer's own output.

use serializetrx::*;

/// Testnet P2WPKH spend.
/// https://blockstream.info/testnet/tx/be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b
const SEGWIT_RAW: &str = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";

/// Block 170, the Satoshi -> Hal Finney transaction. Legacy, two outputs.
/// https://mempool.space/tx/f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16
const LEGACY_RAW: &str = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";

fn serialize(
    version: i32,
    segwit: bool,
    inputs: &[&str],
    outputs: &[&str],
    witnesses: &[&str],
    locktime: u32,
) -> Result<String, SerializeError> {
    let inputs = inputs
        .iter()
        .map(|s| parse_input(s, TxidOrder::Display))
        .collect::<Result<Vec<_>, _>>()?;
    let outputs = outputs
        .iter()
        .map(|s| parse_output(s))
        .collect::<Result<Vec<_>, _>>()?;
    let witnesses = witnesses
        .iter()
        .map(|s| parse_witness(s))
        .collect::<Result<Vec<_>, _>>()?;

    let tx = build_transaction(version, segwit, inputs, outputs, witnesses, locktime)?;
    Ok(bytes_to_hex(&serialize_transaction(&tx)))
}

#[test]
fn reproduces_real_segwit_transaction() {
    let hex = serialize(
        2,
        true,
        &["bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1"],
        &[
            "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
            "4462282:0014599bcef6387256c6b019030c421b4a4d382fe260",
        ],
        &["0:304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c201,020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f1"],
        0,
    )
    .unwrap();

    assert_eq!(hex, SEGWIT_RAW);
}

#[test]
fn reproduces_real_legacy_transaction() {
    let hex = serialize(
        1,
        false,
        &["0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:0:47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901"],
        &[
            "1000000000:4104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac",
            "4000000000:410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac",
        ],
        &[],
        0,
    )
    .unwrap();

    assert_eq!(hex, LEGACY_RAW);
}

#[test]
fn supports_multiple_inputs_and_outputs() {
    let txid = "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9";
    let hex = serialize(
        2,
        false,
        &[&format!("{txid}:0"), &format!("{txid}:1"), &format!("{txid}:2")],
        &["1000:0014274ae586ad2035efb4c25049c155f98310d7e106", "2000:51", "3000:6a"],
        &[],
        0,
    )
    .unwrap();

    // Input count and output count are both 3, as single-byte CompactSize.
    assert!(hex.starts_with("0200000003"), "got {hex}");
    assert_eq!(hex.matches("03e8").count(), 1, "1000 sats appears once");
}

#[test]
fn txid_is_reversed_for_display_order_but_not_internal() {
    let txid = "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9";

    let display = parse_input(&format!("{txid}:0"), TxidOrder::Display).unwrap();
    let internal = parse_input(&format!("{txid}:0"), TxidOrder::Internal).unwrap();

    assert_eq!(bytes_to_hex(&internal.prev_txid), txid);
    assert_ne!(bytes_to_hex(&display.prev_txid), txid);

    let mut reversed = internal.prev_txid.clone();
    reversed.reverse();
    assert_eq!(display.prev_txid, reversed);
}

#[test]
fn input_defaults_are_applied() {
    let input = parse_input(
        "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:7",
        TxidOrder::Display,
    )
    .unwrap();

    assert_eq!(input.vout, 7);
    assert!(input.script_sig.is_empty(), "script_sig defaults to empty");
    assert_eq!(input.sequence, SEQUENCE_FINAL);
}

#[test]
fn numbers_accept_decimal_and_hex() {
    let txid = "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9";

    let decimal = parse_input(&format!("{txid}:0::4294967295"), TxidOrder::Display).unwrap();
    let hex = parse_input(&format!("{txid}:0::0xffffffff"), TxidOrder::Display).unwrap();

    assert_eq!(decimal.sequence, hex.sequence);
    assert_eq!(hex.sequence, SEQUENCE_FINAL);
}

// --- CompactSize boundaries -------------------------------------------------

#[test]
fn varint_encodes_each_width() {
    assert_eq!(encode_varint(0), vec![0x00]);
    assert_eq!(encode_varint(252), vec![0xfc]);
    assert_eq!(encode_varint(253), vec![0xfd, 0xfd, 0x00]);
    assert_eq!(encode_varint(65535), vec![0xfd, 0xff, 0xff]);
    assert_eq!(encode_varint(65536), vec![0xfe, 0x00, 0x00, 0x01, 0x00]);
    assert_eq!(
        encode_varint(4_294_967_296),
        vec![0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
    );
}

#[test]
fn long_script_uses_multi_byte_varint() {
    // A 300-byte script needs the 0xfd form: 300 == 0x012c, little-endian 2c 01.
    let script = "ab".repeat(300);
    let output = parse_output(&format!("1000:{script}")).unwrap();

    assert_eq!(output.script_pubkey.len(), 300);
    assert_eq!(encode_varint(300), vec![0xfd, 0x2c, 0x01]);
}

// --- Validation -------------------------------------------------------------

#[test]
fn rejects_odd_length_hex() {
    let error = hex_to_bytes("abc", "script_sig").unwrap_err();
    assert_eq!(
        error,
        SerializeError::OddLengthHex {
            field: "script_sig".to_string(),
            length: 3
        }
    );
    assert!(error.to_string().contains("even number"));
}

#[test]
fn rejects_non_hex_characters() {
    let error = hex_to_bytes("zz", "script_sig").unwrap_err();
    assert!(matches!(error, SerializeError::InvalidHexDigit { .. }));
    assert!(error.to_string().contains("not valid hexadecimal"));
}

#[test]
fn rejects_txid_of_wrong_length() {
    let error = parse_input("abcd:0", TxidOrder::Display).unwrap_err();
    assert_eq!(error, SerializeError::BadTxidLength { length: 2 });
    assert!(error.to_string().contains("32 bytes"));
}

#[test]
fn rejects_malformed_specs() {
    assert!(matches!(
        parse_input("just-a-txid", TxidOrder::Display).unwrap_err(),
        SerializeError::MalformedSpec { .. }
    ));
    assert!(matches!(
        parse_output("1000").unwrap_err(),
        SerializeError::MalformedSpec { .. }
    ));
    assert!(matches!(
        parse_witness("no-colon").unwrap_err(),
        SerializeError::MalformedSpec { .. }
    ));
}

#[test]
fn rejects_non_numeric_amount() {
    let error = parse_output("lots:51").unwrap_err();
    assert!(matches!(error, SerializeError::NotANumber { .. }));
    assert!(error.to_string().contains("not a valid number"));
}

#[test]
fn rejects_amount_above_supply_cap() {
    let error = parse_output(&format!("{}:51", MAX_MONEY + 1)).unwrap_err();
    assert!(matches!(error, SerializeError::AmountTooLarge { .. }));
    assert!(error.to_string().contains("21 million"));

    // Exactly at the cap is fine.
    assert!(parse_output(&format!("{MAX_MONEY}:51")).is_ok());
}

#[test]
fn rejects_witness_without_segwit() {
    let error = serialize(
        2,
        false,
        &["0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:0"],
        &["1000:51"],
        &["0:aabb"],
        0,
    )
    .unwrap_err();

    assert_eq!(error, SerializeError::WitnessWithoutSegwit);
}

#[test]
fn rejects_segwit_without_witness() {
    // BIP144 forbids the marker and flag when there is no witness data.
    let error = serialize(
        2,
        true,
        &["0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:0"],
        &["1000:51"],
        &[],
        0,
    )
    .unwrap_err();

    assert_eq!(error, SerializeError::SegwitWithoutWitness);
}

#[test]
fn rejects_witness_index_out_of_range() {
    let error = serialize(
        2,
        true,
        &["0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:0"],
        &["1000:51"],
        &["5:aabb"],
        0,
    )
    .unwrap_err();

    assert_eq!(
        error,
        SerializeError::WitnessIndexOutOfRange {
            index: 5,
            input_count: 1
        }
    );
}

#[test]
fn rejects_duplicate_witness_for_same_input() {
    let error = serialize(
        2,
        true,
        &["0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:0"],
        &["1000:51"],
        &["0:aabb", "0:ccdd"],
        0,
    )
    .unwrap_err();

    assert_eq!(error, SerializeError::DuplicateWitness { index: 0 });
}

#[test]
fn segwit_input_without_witness_gets_empty_stack() {
    // Two inputs, witness supplied only for the second. The first must still
    // get a 0x00 count so the witness block lines up with the inputs.
    let txid = "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9";
    let hex = serialize(
        2,
        true,
        &[&format!("{txid}:0"), &format!("{txid}:1")],
        &["1000:51"],
        &["1:aabb"],
        0,
    )
    .unwrap();

    // Witness block: 0x00 (empty stack for input 0), then 0x01 0x02 aabb.
    assert!(hex.contains("000102aabb"), "got {hex}");
}
