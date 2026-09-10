use bitcoin_tx_serializer::{
    bytes_to_hex, hex_to_bytes, parse_input, parse_output, assign_witness_data, run, Args, TxInput,
};

#[test]
fn test_parse_input_minimal() {
    // Format: prev_txid:vout
    let input_str = "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:0";
    let result = parse_input(input_str, 0);

    assert!(result.is_ok());
    let input = result.unwrap();
    assert_eq!(input.vout, 0);
    assert_eq!(input.sequence, 0xffffffff);
    assert!(input.script_sig.is_empty());
}

#[test]
fn test_parse_input_with_script_sig() {
    // Format: prev_txid:vout:script_sig
    let input_str = "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:0:483045022100abcd";
    let result = parse_input(input_str, 0);

    assert!(result.is_ok());
    let input = result.unwrap();
    assert_eq!(input.vout, 0);
    assert_eq!(input.sequence, 0xffffffff);
    assert_eq!(bytes_to_hex(&input.script_sig), "483045022100abcd");
}

#[test]
fn test_parse_input_with_sequence() {
    // Format: prev_txid:vout:script_sig:sequence
    let input_str = "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:0:483045022100abcd:fffffffe";
    let result = parse_input(input_str, 0);

    assert!(result.is_ok());
    let input = result.unwrap();
    assert_eq!(input.vout, 0);
    assert_eq!(input.sequence, 0xfffffffe);
}

#[test]
fn test_parse_input_empty_script_sig() {
    // Format: prev_txid:vout::sequence (empty script_sig)
    let input_str = "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:0::fffffffe";
    let result = parse_input(input_str, 0);

    assert!(result.is_ok());
    let input = result.unwrap();
    assert!(input.script_sig.is_empty());
    assert_eq!(input.sequence, 0xfffffffe);
}

#[test]
fn test_parse_input_invalid_format_missing_vout() {
    let input_str = "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a";
    let result = parse_input(input_str, 0);

    assert!(result.is_err());
}

#[test]
fn test_parse_input_invalid_txid_length() {
    // TXID must be 64 hex characters (32 bytes)
    let input_str = "abcd:0";
    let result = parse_input(input_str, 0);

    assert!(result.is_err());
}

#[test]
fn test_parse_input_invalid_vout() {
    let input_str = "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:abc";
    let result = parse_input(input_str, 0);

    assert!(result.is_err());
}

#[test]
fn test_parse_input_invalid_hex_script_sig() {
    let input_str = "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:0:zzzzzz";
    let result = parse_input(input_str, 0);

    assert!(result.is_err());
}

#[test]
fn test_parse_input_invalid_sequence() {
    let input_str = "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:0:483045022100abcd:gggggg";
    let result = parse_input(input_str, 0);

    assert!(result.is_err());
}

#[test]
fn test_parse_input_multiple_vouts() {
    // Test with different vout values
    for vout in &[0, 1, 100, 1000] {
        let input_str = format!(
            "d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:{}",
            vout
        );
        let result = parse_input(&input_str, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().vout, *vout);
    }
}

#[test]
fn test_parse_output_valid() {
    let output_str = "95000:76a91412ab34cd56ef78901234567890abcdef12345678ac";
    let result = parse_output(output_str);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.value, 95000);
    assert_eq!(
        bytes_to_hex(&output.script_pubkey),
        "76a91412ab34cd56ef78901234567890abcdef12345678ac"
    );
}

#[test]
fn test_parse_output_zero_value() {
    let output_str = "0:76a914";
    let result = parse_output(output_str);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.value, 0);
}

#[test]
fn test_parse_output_large_value() {
    let output_str = "2100000000000000:76a914";
    let result = parse_output(output_str);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.value, 2100000000000000);
}

#[test]
fn test_parse_output_empty_script() {
    let output_str = "50000:";
    let result = parse_output(output_str);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.value, 50000);
    assert!(output.script_pubkey.is_empty());
}

#[test]
fn test_parse_output_invalid_format_no_colon() {
    let output_str = "95000";
    let result = parse_output(output_str);

    assert!(result.is_err());
}

#[test]
fn test_parse_output_invalid_format_too_many_colons() {
    let output_str = "95000:76a914:extra";
    let result = parse_output(output_str);

    assert!(result.is_err());
}

#[test]
fn test_parse_output_invalid_value() {
    let output_str = "abc:76a914";
    let result = parse_output(output_str);

    assert!(result.is_err());
}

#[test]
fn test_parse_output_invalid_hex_script() {
    let output_str = "95000:zzzzzz";
    let result = parse_output(output_str);

    assert!(result.is_err());
}

#[test]
fn test_assign_witness_data_single_input_single_item() {
    let mut input = TxInput::new(
        hex_to_bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let mut inputs = vec![input];
    let witness_items = vec!["483045022100abcd".to_string()];

    let result = assign_witness_data(&mut inputs, &witness_items, &None);

    assert!(result.is_ok());
    assert_eq!(inputs[0].witness.len(), 1);
    assert_eq!(bytes_to_hex(&inputs[0].witness[0]), "483045022100abcd");
}

#[test]
fn test_assign_witness_data_multiple_inputs_with_counts() {
    let input1 = TxInput::new(
        hex_to_bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let input2 = TxInput::new(
        hex_to_bytes("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let mut inputs = vec![input1, input2];
    let witness_items = vec![
        "483045022100aaaa".to_string(),
        "483045022100bbbb".to_string(),
        "483045022100cccc".to_string(),
    ];
    let witness_counts = Some("2,1".to_string());

    let result = assign_witness_data(&mut inputs, &witness_items, &witness_counts);

    assert!(result.is_ok());
    assert_eq!(inputs[0].witness.len(), 2);
    assert_eq!(inputs[1].witness.len(), 1);
    assert_eq!(bytes_to_hex(&inputs[0].witness[0]), "483045022100aaaa");
    assert_eq!(bytes_to_hex(&inputs[0].witness[1]), "483045022100bbbb");
    assert_eq!(bytes_to_hex(&inputs[1].witness[0]), "483045022100cccc");
}

#[test]
fn test_assign_witness_data_no_witness_items() {
    let mut input = TxInput::new(
        hex_to_bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let mut inputs = vec![input];
    let witness_items: Vec<String> = Vec::new();

    let result = assign_witness_data(&mut inputs, &witness_items, &None);

    assert!(result.is_err());
}

#[test]
fn test_assign_witness_data_count_mismatch() {
    let input1 = TxInput::new(
        hex_to_bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let input2 = TxInput::new(
        hex_to_bytes("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let mut inputs = vec![input1, input2];
    let witness_items = vec!["483045022100aaaa".to_string()];
    let witness_counts = Some("2,1".to_string()); // expects 3 items, but only 1 provided

    let result = assign_witness_data(&mut inputs, &witness_items, &witness_counts);

    assert!(result.is_err());
}

#[test]
fn test_assign_witness_data_unused_items() {
    let mut input = TxInput::new(
        hex_to_bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let mut inputs = vec![input];
    let witness_items = vec![
        "483045022100aaaa".to_string(),
        "483045022100bbbb".to_string(), // This won't be used
    ];
    let witness_counts = Some("1".to_string());

    let result = assign_witness_data(&mut inputs, &witness_items, &witness_counts);

    assert!(result.is_err());
}

#[test]
fn test_assign_witness_data_invalid_count_format() {
    let mut input = TxInput::new(
        hex_to_bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let mut inputs = vec![input];
    let witness_items = vec!["483045022100aaaa".to_string()];
    let witness_counts = Some("abc".to_string());

    let result = assign_witness_data(&mut inputs, &witness_items, &witness_counts);

    assert!(result.is_err());
}

#[test]
fn test_assign_witness_data_invalid_hex() {
    let mut input = TxInput::new(
        hex_to_bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        0,
        Vec::new(),
        0xffffffff,
    )
    .unwrap();

    let mut inputs = vec![input];
    let witness_items = vec!["zzzzzz".to_string()]; // Invalid hex

    let result = assign_witness_data(&mut inputs, &witness_items, &None);

    assert!(result.is_err());
}

#[test]
fn test_run_minimal_legacy_transaction() {
    let args = Args {
        tx_version: 2,
        segwit: false,
        input: vec!["d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:0".to_string()],
        output: vec!["95000:76a91412ab34cd56ef78901234567890abcdef12345678ac".to_string()],
        witness: Vec::new(),
        witness_counts: None,
        locktime: 0,
    };

    let result = run(args);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Transaction Details:"));
    assert!(output.contains("Version:                  2"));
    assert!(output.contains("Serialized Transaction (Hex):"));
}

#[test]
fn test_run_legacy_transaction_with_script_sig() {
    let args = Args {
        tx_version: 2,
        segwit: false,
        input: vec!["d6892c90f188fe1991c949850c6af6240034b9b3a8918f5944c3c6d93e0e8d7a:0:483045022100abcd".to_string()],
        output: vec!["95000:76a91412ab34cd56ef78901234567890abcdef12345678ac".to_string()],
        witness: Vec::new(),
        witness_counts: None,
        locktime: 0,
    };

    let result = run(args);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("ScriptSig:"));
}

