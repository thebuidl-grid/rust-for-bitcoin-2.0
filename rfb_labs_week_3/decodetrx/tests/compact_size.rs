use decodetrx::{error::DecodeError, read_compact_size};

#[test]
fn reads_a_single_byte_compact_size() {
    let mut bytes: &[u8] = &[0x01, 0xaa];

    let value = read_compact_size(&mut bytes).unwrap();

    assert_eq!(value, 1);
    assert_eq!(bytes, &[0xaa]);
}

#[test]
fn reads_a_u16_compact_size() {
    let mut bytes: &[u8] = &[0xfd, 0x00, 0x01, 0xaa];

    let value = read_compact_size(&mut bytes).unwrap();

    assert_eq!(value, 256);
    assert_eq!(bytes, &[0xaa]);
}

#[test]
fn reads_a_u32_compact_size() {
    let mut bytes: &[u8] = &[0xfe, 0x00, 0x00, 0x01, 0x00, 0xaa];

    let value = read_compact_size(&mut bytes).unwrap();

    assert_eq!(value, 65_536);
    assert_eq!(bytes, &[0xaa]);
}

#[test]
fn reads_a_u64_compact_size() {
    let mut bytes: &[u8] = &[0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xaa];

    let value = read_compact_size(&mut bytes).unwrap();

    assert_eq!(value, 4_294_967_296);
    assert_eq!(bytes, &[0xaa]);
}

#[test]
fn rejects_truncated_compact_size_data() {
    let mut bytes: &[u8] = &[0xfd, 0x01];

    let error = read_compact_size(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}

#[test]
fn rejects_a_noncanonical_compact_size() {
    let mut bytes: &[u8] = &[0xfd, 0xfc, 0x00];

    let error = read_compact_size(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::InvalidCompactSize));
}
