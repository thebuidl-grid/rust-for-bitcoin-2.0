use decodetrx::decode_transaction;
use serde_json::Value;

const GENESIS_COINBASE_TRANSACTION: &str = concat!(
    "01000000",
    "01",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "ffffffff",
    "4d",
    "04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73",
    "ffffffff",
    "01",
    "00f2052a01000000",
    "43",
    "4104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac",
    "00000000",
);

const SEGWIT_TRANSACTION: &str = "02000000000101af032baef1838ce072e0541a29a700127edf80298c4136e99371ac4a6f2231ec0000000000fdffffff02813e0000000000001600145598af355466d36e6a671374abbb2b75ef4779560000000000000000156a5d1214011400ff7f818cec82d08bc0a88281d21502473044022007ecae7dfd412adebece20f97dba8fbcd9bc1c495ef2213604a320dd5371f587022058d218c56d5acf4a0e39a53eda36e1d8b2b9967c8a4bb1cde6e0f7c1f2a6cc090121037b72b3c3f3d7e4107fa7a08ecffd74dc16ee332abe6eafabd89acea8bdc66f4200000000";

#[test]
fn decodes_a_legacy_transaction_and_calculates_its_txid() {
    let decoded = decode_transaction(GENESIS_COINBASE_TRANSACTION.to_owned()).unwrap();
    let transaction: Value = serde_json::from_str(&decoded).unwrap();

    assert_eq!(
        transaction["transaction_id"],
        "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"
    );
    assert_eq!(transaction["version"], 1);
    assert_eq!(transaction["inputs"].as_array().unwrap().len(), 1);
    assert_eq!(transaction["inputs"][0]["output_index"], u32::MAX);
    assert!(transaction["inputs"][0].get("witness").is_none());
    assert_eq!(transaction["outputs"].as_array().unwrap().len(), 1);
    assert_eq!(transaction["outputs"][0]["amount"], 50.0);
    assert_eq!(transaction["lock_time"], 0);
}

#[test]
fn decodes_a_segwit_transaction_and_its_witnesses() {
    let decoded = decode_transaction(SEGWIT_TRANSACTION.to_owned()).unwrap();
    let transaction: Value = serde_json::from_str(&decoded).unwrap();

    assert_eq!(transaction["version"], 2);
    assert_eq!(transaction["inputs"].as_array().unwrap().len(), 1);
    assert_eq!(transaction["inputs"][0]["script_sig"], "");
    assert_eq!(
        transaction["inputs"][0]["witness"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(transaction["outputs"].as_array().unwrap().len(), 2);
    assert_eq!(transaction["outputs"][0]["amount"], 0.00016001);
    assert_eq!(transaction["outputs"][1]["amount"], 0.0);
    assert_eq!(transaction["lock_time"], 0);
}

#[test]
fn rejects_invalid_hex() {
    let error = decode_transaction("not-hex".to_owned()).unwrap_err();

    assert!(error.to_string().contains("invalid transaction hex"));
}

#[test]
fn rejects_unparsed_trailing_bytes() {
    let transaction_with_trailing_byte = format!("{GENESIS_COINBASE_TRANSACTION}aa");

    let error = decode_transaction(transaction_with_trailing_byte).unwrap_err();

    assert!(error.to_string().contains("1 unparsed trailing byte"));
}
