use decodetrx::{error::DecodeError, read_script_size};

#[test]
fn reads_a_compact_size_prefixed_script_as_bytes() {
    let mut bytes: &[u8] = &[0x03, 0xaa, 0xbb, 0xcc, 0xdd];

    let script = read_script_size(&mut bytes).unwrap();

    assert_eq!(script, vec![0xaa, 0xbb, 0xcc]);
    assert_eq!(bytes, &[0xdd]);
}

#[test]
fn reads_an_empty_script() {
    let mut bytes: &[u8] = &[0x00, 0xdd];

    let script = read_script_size(&mut bytes).unwrap();

    assert!(script.is_empty());
    assert_eq!(bytes, &[0xdd]);
}

#[test]
fn rejects_a_script_shorter_than_its_declared_size() {
    let mut bytes: &[u8] = &[0x03, 0xaa];

    let error = read_script_size(&mut bytes).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}
