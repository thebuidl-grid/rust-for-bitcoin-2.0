use decodetrx::{error::DecodeError, read_u32};

#[test]
fn reads_little_endian_u32_and_advances_input() {
    let mut bytes: &[u8] = &[0x02, 0x00, 0x00, 0x00, 0xaa, 0xbb];

    let value = read_u32(&mut bytes).unwrap();

    assert_eq!(value, 2);
    assert_eq!(bytes, &[0xaa, 0xbb]);
}

#[test]
fn reading_u32_from_too_few_bytes_returns_io_error() {
    let mut bytes: &[u8] = &[0x02, 0x00, 0x00];

    let error = read_u32(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}
