#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_valid_testnet_transaction() {
        // A valid 1-input, 2-output legacy testnet transaction hex
        let raw_hex = "010000000180a37e543666ffbd284f1eb8ec480b55edcfefefbe311fb924e2c842b083c26a000000006b483045022100cb0cb38f45a0ed3f3d79383824300bfb2d41fb377033cb2462bd87dbcf5e064602206ffad99f36340dbdbbf92ff1598f8a1e3eeef5a92a7f0e9f086b9a8964ff03990121035b443261623910c8bc032c1033aa7fb31cb02c8928014522a57eb05799a4e32dffffff202029013000000001976a914cda640304a08ef48d886ffdd0f40a430ff2a7e7188acb8001000000000001976a9143ba7fb50bf088cb6228f4ff936ca4ee0723ff43288ac00000000";

        // Attempt to decode the transaction
        let result = decode_transaction(raw_hex.to_string());
        
        // Assert that decoding succeeded
        assert!(result.is_ok(), "Expected transaction to decode successfully");
        
        let json_str = result.unwrap();
        
        // Parse the returned JSON back into serde_json::Value to verify key fields
        let json_val: serde_json::Value = serde_json::from_str(&json_str)
            .expect("Returned string should be valid JSON");

        // 1. Verify Transaction Version
        assert_eq!(json_val["version"], 1);

        // 2. Verify Locktime
        assert_eq!(json_val["lock_time"], 0);

        // 3. Verify Inputs Count
        let inputs = json_val["inputs"].as_array().expect("Inputs should be an array");
        assert_eq!(inputs.len(), 1);

        // Verify First Input Output Index
        assert_eq!(inputs[0]["output_index"], 0);

        // 4. Verify Outputs Count
        let outputs = json_val["outputs"].as_array().expect("Outputs should be an array");
        assert_eq!(outputs.len(), 2);

        // Verify Output Amounts (serialized as BTC float by our custom `as_btc` function)
        let output_0_amount = outputs[0]["amount"].as_f64().unwrap();
        let output_1_amount = outputs[1]["amount"].as_f64().unwrap();

        // 320,000,000 sats = 3.2 BTC
        assert_eq!(output_0_amount, 3.2);
        // 1,048,760 sats = 0.0104876 BTC
        assert_eq!(output_1_amount, 0.0104876);
    }

    #[test]
    fn test_decode_invalid_hex_fails() {
        let invalid_hex = "010000000180a37e543666ffbd284f1eb8ec480b55edINVALID_HEX";
        let result = decode_transaction(invalid_hex.to_string());
        
        // Decoding should fail cleanly and return an Error
        assert!(result.is_err());
    }
}