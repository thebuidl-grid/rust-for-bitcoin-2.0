use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

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

// These tests are ignored so the starter repository builds before students
// implement the TODOs. Remove `#[ignore]` from one test at a time while working.

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

#[test]
fn outputs_cannot_exceed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_output(output(50_000, "bc1qreceiver"));
    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn highest_output_returns_correct_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(100_000));
    transaction.add_output(output(30_000, "a"));
    transaction.add_output(output(50_000, "b"));
    transaction.add_output(output(10_000, "c"));

    let highest = rfb_labs_week_2::highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 50_000);
    assert_eq!(highest.recipient, "b");
}

#[test]
fn find_outputs_for_recipient_works() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(100_000));
    transaction.add_output(output(30_000, "a"));
    transaction.add_output(output(50_000, "b"));
    transaction.add_output(output(10_000, "a"));

    let outputs_for_a = rfb_labs_week_2::find_outputs_for_recipient(&transaction, "a");
    assert_eq!(outputs_for_a.len(), 2);
    assert_eq!(outputs_for_a[0].value, 30_000);
    assert_eq!(outputs_for_a[1].value, 10_000);
}

#[test]
fn validation_error_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(50_000, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn validation_error_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn validation_error_zero_value_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));

    // non-OpReturn zero output should fail
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::ZeroValueOutput)
    );

    // OpReturn zero output should pass
    let mut transaction_op_return = Transaction::new(2, 0);
    transaction_op_return.add_input(regular_input(50_000));
    transaction_op_return.add_output(TxOutput {
        value: 0,
        recipient: "".into(),
        output_type: OutputType::OpReturn,
    });
    assert_eq!(transaction_op_return.validate(), Ok(()));
}

#[test]
fn validation_error_mixed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_output(output(80_000, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn validation_error_multiple_coinbase() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_input(InputKind::Coinbase {
        block_height: 101,
        reward: 50_000,
    });
    transaction.add_output(output(80_000, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn validation_error_invalid_txid() {
    // Invalid characters
    let mut tx = Transaction::new(2, 0);
    tx.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "invalidtxidcharacters_g_z_nothex_000000000000000000000000000".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    tx.add_output(output(40_000, "bc1qreceiver"));
    assert_eq!(
        tx.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );

    // Wrong length
    let mut tx_short = Transaction::new(2, 0);
    tx_short.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "abc123ef".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    tx_short.add_output(output(40_000, "bc1qreceiver"));
    assert_eq!(
        tx_short.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}

#[test]
fn transaction_display_format() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    let display_str = format!("{}", transaction);
    assert!(display_str.contains("Transaction (v2, locktime: 0)"));
    assert!(display_str.contains("Inputs (count: 1)"));
    assert!(display_str.contains("Outputs (count: 1)"));
    assert!(display_str.contains("Total Input:  120000 sats"));
    assert!(display_str.contains("Total Output: 90000 sats"));
    assert!(display_str.contains("Calculated Fee: 30000 sats"));
}
