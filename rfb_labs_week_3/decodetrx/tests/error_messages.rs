use std::io;

use decodetrx::error::DecodeError;

#[test]
fn every_decode_error_has_a_readable_message() {
    let invalid_hex = DecodeError::InvalidHex(hex::decode("0").unwrap_err());
    assert_eq!(
        invalid_hex.to_string(),
        "invalid transaction hex: Odd number of digits"
    );

    let io_error = DecodeError::Io(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "transaction ended early",
    ));
    assert_eq!(
        io_error.to_string(),
        "transaction data is incomplete: expected more bytes"
    );

    assert_eq!(
        DecodeError::InvalidCompactSize.to_string(),
        "invalid CompactSize encoding: value is not encoded in its shortest form"
    );
    assert_eq!(
        DecodeError::ScriptLengthTooLarge(1_000).to_string(),
        "script length 1000 cannot be represented on this platform"
    );
    assert_eq!(
        DecodeError::InvalidSegwitMarker.to_string(),
        "invalid SegWit marker and flag: expected 00 01"
    );
    assert_eq!(
        DecodeError::TrailingBytes(3).to_string(),
        "transaction contains 3 unparsed trailing byte(s)"
    );

    let serde_error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let serialization_error = DecodeError::Serialization(serde_error);
    assert!(
        serialization_error
            .to_string()
            .starts_with("failed to serialize the decoded transaction:")
    );
}
