use decodetrx::{Txid, error::DecodeError, read_txid};

#[test]
fn reads_32_txid_bytes_and_advances_input() {
    let expected_bytes = [0x11_u8; 32];
    let mut serialized_bytes = expected_bytes.to_vec();
    serialized_bytes.push(0xaa);
    let mut cursor = serialized_bytes.as_slice();

    let txid = read_txid(&mut cursor).unwrap();

    assert_eq!(txid, Txid::from_bytes(expected_bytes));
    assert_eq!(cursor, &[0xaa]);
}

#[test]
fn rejects_a_truncated_txid() {
    let mut cursor: &[u8] = &[0x11; 31];

    let error = read_txid(&mut cursor).unwrap_err();

    assert!(matches!(error, DecodeError::Io(_)));
}
