use decodetrx::{error::DecodeError, read_bytes};

#[test]
fn reads_the_requested_number_of_bytes_and_advances_input() {
    let mut bytes: &[u8] = &[0xaa, 0xbb, 0xcc, 0xdd];

    let value = read_bytes(&mut bytes, 3).unwrap();

    assert_eq!(value, vec![0xaa, 0xbb, 0xcc]);
    assert_eq!(bytes, &[0xdd]);
}

#[test]
fn reading_zero_bytes_returns_an_empty_vector() {
    let mut bytes: &[u8] = &[0xaa];

    let value = read_bytes(&mut bytes, 0).unwrap();

    assert!(value.is_empty());
    assert_eq!(bytes, &[0xaa]);
}

#[test]
fn rejects_input_shorter_than_the_requested_length() {
    let mut bytes: &[u8] = &[0xaa];

    let error = read_bytes(&mut bytes, 2).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}
