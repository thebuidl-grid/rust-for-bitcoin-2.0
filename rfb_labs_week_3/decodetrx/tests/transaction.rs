use decodetrx::decode_transaction;

// ┌──────────────────────────────┐
// │ Version          4 bytes     │
// ├──────────────────────────────┤
// │ Marker           1 byte      │
// │ Flag             1 byte      │
// ├──────────────────────────────┤
// │ Input count      VarInt      │
// │ Inputs           Variable    │
// ├──────────────────────────────┤
// │ Output count     VarInt      │
// │ Outputs          Variable    │
// ├──────────────────────────────┤
// │ Witness          Variable    │
// ├──────────────────────────────┤
// │ Locktime         4 bytes  ←  │
// └──────────────────────────────┘

#[test]
fn test_decode_valid_testnet_transaction() {
    // We construct the string line-by-line to prevent copy/paste truncation
    let raw_hex = String::new()
        + "01000000" // Version: 1 (4 bytes)
        + "01" // Input Count: 1 (1 byte)
        + "80a37e543666ffbd284f1eb8ec480b55edcfefefbe311fb924e2c842b083c26a" // TxID (32 bytes)
        + "00000000" // Vout: 0 (4 bytes)
        + "6b" // ScriptSig length: 107 bytes (1 byte)
        + "483045022100cb0cb38f45a0ed3f3d79383824300bfb2d41fb377033cb2462bd87dbcf5e064602206ffad99f36340dbdbbf92ff1598f8a1e3eeef5a92a7f0e9f086b9a8964ff03990121035b443261623910c8bc032c1033aa7fb31cb02c8928014522a57eb05799a4e32d" // ScriptSig
        + "ffffffff" // Sequence (4 bytes)
        + "02" // Output Count: 2 (1 byte)
        + "00d0121300000000" // Output 1 Amount: 3.2 BTC in Little Endian (8 bytes)
        + "1976a914cda640304a08ef48d886ffdd0f40a430ff2a7e7188ac" // Output 1 ScriptPubkey
        + "b800100000000000" // Output 2 Amount: 0.0104876 BTC in Little Endian (8 bytes)
        + "1976a9143ba7fb50bf088cb6228f4ff936ca4ee0723ff43288ac" // Output 2 ScriptPubkey
        + "00000000"; // Locktime: 0 (4 bytes)

    let result = decode_transaction(raw_hex.clone());
    assert!(result.is_ok(), "Failed to decode transaction: {:?}", result.err());

    let json_str = result.unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&json_str)
        .expect("Returned string should be valid JSON");

    assert_eq!(json_val["version"], 1);
    assert_eq!(json_val["lock_time"], 0);

    let inputs = json_val["inputs"].as_array().expect("Inputs should be an array");
    assert_eq!(inputs.len(), 1);

    let outputs = json_val["outputs"].as_array().expect("Outputs should be an array");
    assert_eq!(outputs.len(), 2);

    let output_0_amount = outputs[0]["amount"].as_f64().unwrap();
    let output_1_amount = outputs[1]["amount"].as_f64().unwrap();

    assert_eq!(output_0_amount, 3.2);
    assert_eq!(output_1_amount, 0.0104876);
}

#[test]
fn test_decode_invalid_hex_fails() {
    let invalid_hex = "010000000180a37e543666ffbd284f1eb8ec480b55edINVALID_HEX";
    assert!(decode_transaction(invalid_hex.to_string()).is_err());
}