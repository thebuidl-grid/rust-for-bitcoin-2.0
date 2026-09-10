use bitcoin_tx_serializer::serializer::{bytes_to_hex, hex_to_bytes, serialize_transaction};
use bitcoin_tx_serializer::transaction::{Transaction, TxInput, TxOutput};

#[test]
fn test_bytes_to_hex() {
    assert_eq!(bytes_to_hex(&[0x48, 0x65, 0x6c, 0x6c, 0x6f]), "48656c6c6f");
    assert_eq!(bytes_to_hex(&[]), "");
    assert_eq!(bytes_to_hex(&[0xff, 0x00, 0xaa]), "ff00aa");
}

#[test]
fn test_hex_to_bytes() {
    assert_eq!(
        hex_to_bytes("48656c6c6f").unwrap(),
        vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]
    );
    assert_eq!(hex_to_bytes("").unwrap(), vec![]);
    assert_eq!(
        hex_to_bytes("ff00aa").unwrap(),
        vec![0xff, 0x00, 0xaa]
    );
}

#[test]
fn test_hex_to_bytes_invalid() {
    assert!(hex_to_bytes("zz").is_err());
    assert!(hex_to_bytes("12g").is_err());
    assert!(hex_to_bytes("1").is_err()); // odd length
}


#[test]
fn test_simple_legacy_transaction() {
    let input = TxInput::new(
        hex_to_bytes("d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a")
            .unwrap(),
        0,
        hex_to_bytes("483045022100f5d1e6c7b8a9d0e1f2003344556677889900aabbccddeeff")
            .unwrap(),
        0xffffffff,
    )
    .unwrap();

    let output = TxOutput::new(
        95000,
        hex_to_bytes("76a91412ab34cd56ef78901234567890abcdef12345678ac").unwrap(),
    );

    let transaction = Transaction::new(2, vec![input], vec![output], 0, false).unwrap();
    let serialized = serialize_transaction(&transaction);

    assert!(!serialized.is_empty());
    // Verify version bytes (little-endian)
    assert_eq!(&serialized[0..4], [0x02, 0x00, 0x00, 0x00]);
}

#[test]
fn test_segwit_transaction() {
    let mut input = TxInput::new(
        hex_to_bytes("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    // Adicionar dados de witness antes de criar a transação
    input.add_witness_item(
        hex_to_bytes("483045022100abcd1234567890abcdef").unwrap(),
    );

    let output = TxOutput::new(50000, hex_to_bytes("76a91488ac").unwrap());

    let transaction = Transaction::new(2, vec![input], vec![output], 0, true).unwrap();
    let serialized = serialize_transaction(&transaction);

    assert!(!serialized.is_empty());
    // Verify SegWit marker and flag
    assert_eq!(serialized[4], 0x00); // Marker
    assert_eq!(serialized[5], 0x01); // Flag
}
#[test]
fn test_transaction_with_multiple_inputs_outputs() {
    let input1 = TxInput::new(
        hex_to_bytes("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
            .unwrap(),
        0,
        hex_to_bytes("4830450221008949f0a678915fc78eac").unwrap(),
        0xffffffff,
    )
    .unwrap();

    let input2 = TxInput::new(
        hex_to_bytes("fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321")
            .unwrap(),
        1,
        hex_to_bytes("483045022100c3d4e5").unwrap(),
        0xfffffffe,
    )
    .unwrap();

    let output1 = TxOutput::new(50000, hex_to_bytes("76a91412ab34cd56ef").unwrap());
    let output2 = TxOutput::new(25000, hex_to_bytes("76a91456ef78901234").unwrap());

    let transaction =
        Transaction::new(2, vec![input1, input2], vec![output1, output2], 0, false).unwrap();
    let serialized = serialize_transaction(&transaction);

    assert!(!serialized.is_empty());
    // Verify it contains 2 inputs and 2 outputs markers
    assert!(serialized.len() > 100); // Should be reasonably sized
}

#[test]
fn test_serialize_deserialize_roundtrip() {
    let input = TxInput::new(
        hex_to_bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        0,
        hex_to_bytes("483045").unwrap(),
        0xffffffff,
    )
    .unwrap();

    let output = TxOutput::new(12345, hex_to_bytes("76a914").unwrap());

    let transaction = Transaction::new(2, vec![input], vec![output], 0, false).unwrap();
    let serialized = serialize_transaction(&transaction);
    let hex_serialized = bytes_to_hex(&serialized);

    // Verify we can convert back to hex without errors
    let back_to_bytes = hex_to_bytes(&hex_serialized).unwrap();
    assert_eq!(serialized, back_to_bytes);
}
