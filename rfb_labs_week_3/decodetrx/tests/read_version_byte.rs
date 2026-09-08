use decodetrx::{error::DecodeError, read_version_byte};

#[test]
fn reads_version_from_decoded_bytes_and_advances_input() {
    let mut bytes: &[u8] = &[0x02, 0x00, 0x00, 0x00, 0xaa];

    let version = read_version_byte(&mut bytes).unwrap();

    assert_eq!(version, 2);
    assert_eq!(bytes, &[0xaa]);
}

#[test]
fn rejects_a_truncated_version() {
    let mut bytes: &[u8] = &[0x02, 0x00, 0x00];

    let error = read_version_byte(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}
