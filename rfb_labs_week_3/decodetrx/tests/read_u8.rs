use decodetrx::{error::DecodeError, read_u8};

#[test]
fn reads_one_byte_at_a_time_and_advances_input() {
    let mut bytes: &[u8] = &[0x00, 0x01, 0xaa];

    let marker = read_u8(&mut bytes).unwrap();
    let flag = read_u8(&mut bytes).unwrap();

    assert_eq!(marker, 0x00);
    assert_eq!(flag, 0x01);
    assert_eq!(bytes, &[0xaa]);
}

#[test]
fn reading_u8_from_empty_input_returns_io_error() {
    let mut bytes: &[u8] = &[];

    let error = read_u8(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}
