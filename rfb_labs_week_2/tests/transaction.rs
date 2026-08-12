use rfb_labs_week_2::{
    find_outputs_for_recipient, highest_value_output, InputKind, OutPoint, OutputType, Transaction,
    TransactionError, TxOutput,
};

fn regular_input(value: u64) -> InputKind {
    InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value,
        sequence: u32::MAX,
    }
}

fn output(value: u64, recipient: &str) -> TxOutput {
    TxOutput {
        value,
        recipient: recipient.into(),
        output_type: OutputType::P2wpkh,
    }
}

// 1. Valid Regular Transaction
#[test]
fn valid_regular_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 120_000);
    assert_eq!(transaction.total_output_value(), 118_000);
    assert_eq!(transaction.fee(), Ok(2_000));
}

// 2. Highest Output & Recipient Filtering

#[test]
fn test_highest_value_output() {
    // 1. Case with multiple outputs of varying values
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(200_000));

    transaction.add_output(output(20_000, "bc1qsmall"));
    transaction.add_output(output(110_000, "bc1qlargest"));
    transaction.add_output(output(50_000, "bc1qmedium"));

    let highest = highest_value_output(&transaction);
    assert!(highest.is_some());
    let highest = highest.unwrap();
    assert_eq!(highest.value, 110_000);
    assert_eq!(highest.recipient, "bc1qlargest");

    // 2. Case with empty outputs (should return None)
    let empty_tx = Transaction::new(2, 0);
    assert_eq!(highest_value_output(&empty_tx), None);
}

#[test]
fn test_find_outputs_for_recipient() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(150_000));

    // Add multiple outputs to different recipients
    transaction.add_output(output(50_000, "bc1qalice"));
    transaction.add_output(output(40_000, "bc1qbob"));
    transaction.add_output(output(30_000, "bc1qalice")); // Second output for Alice

    // Query for Alice (should find 2 outputs)
    let alice_outputs = find_outputs_for_recipient(&transaction, "bc1qalice");
    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs[0].value, 50_000);
    assert_eq!(alice_outputs[1].value, 30_000);

    // Query for Bob (should find 1 output)
    let bob_outputs = find_outputs_for_recipient(&transaction, "bc1qbob");
    assert_eq!(bob_outputs.len(), 1);
    assert_eq!(bob_outputs[0].value, 40_000);

    // Query for non-existent recipient (should return an empty Vec)
    let unknown_outputs = find_outputs_for_recipient(&transaction, "bc1qcharlie");
    assert!(unknown_outputs.is_empty());
}

// 3. Valid Coinbase Transaction
#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(1, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 840_000,
        reward: 3_125_000,
    });
    transaction.add_output(output(3_125_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
}

// 4. Validation Error: Outputs Exceed Inputs
#[test]
fn outputs_cannot_exceed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

// 5. Validation Error: No Inputs
#[test]
fn transaction_must_have_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

// 6. Validation Error: No Outputs
#[test]
fn transaction_must_have_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

// 7. Validation Error: Mixed Input Types
#[test]
fn cannot_mix_coinbase_and_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

// 8. Validation Error: Zero Value Output
#[test]
fn non_op_return_outputs_must_be_positive() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}
