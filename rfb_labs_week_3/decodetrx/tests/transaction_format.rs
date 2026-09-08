use decodetrx::{TransactionFormat, error::DecodeError, read_transaction_format};

#[test]
fn recognizes_legacy_without_consuming_input_count() {
    let mut bytes: &[u8] = &[0x02, 0xaa];

    let format = read_transaction_format(&mut bytes).unwrap();

    assert_eq!(format, TransactionFormat::Legacy);
    assert_eq!(bytes, &[0x02, 0xaa]);
}

#[test]
fn recognizes_segwit_and_consumes_marker_and_flag() {
    let mut bytes: &[u8] = &[0x00, 0x01, 0x02, 0xaa];

    let format = read_transaction_format(&mut bytes).unwrap();

    assert_eq!(format, TransactionFormat::Segwit);
    assert_eq!(bytes, &[0x02, 0xaa]);
}

#[test]
fn rejects_an_invalid_segwit_flag() {
    let mut bytes: &[u8] = &[0x00, 0x02, 0xaa];

    let error = read_transaction_format(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::InvalidSegwitMarker));
}

#[test]
fn rejects_a_truncated_segwit_prefix() {
    let mut bytes: &[u8] = &[0x00];

    let error = read_transaction_format(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}
