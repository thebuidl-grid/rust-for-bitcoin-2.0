use decodetrx::{error::DecodeError, read_amount};

#[test]
fn reads_a_little_endian_satoshi_amount() {
    let mut bytes: &[u8] = &[0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xaa];

    let amount = read_amount(&mut bytes).unwrap();

    assert_eq!(amount.to_sat(), 100);
    assert_eq!(bytes, &[0xaa]);
}

#[test]
fn rejects_a_truncated_amount() {
    let mut bytes: &[u8] = &[0x64, 0x00, 0x00];

    let error = read_amount(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}
