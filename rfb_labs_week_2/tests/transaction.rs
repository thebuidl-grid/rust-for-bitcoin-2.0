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
fn highest_output_returns_the_largest_value() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "alice"));
    transaction.add_output(output(50_000, "bob"));
    transaction.add_output(output(20_000, "charlie"));

    let highest = rfb_labs_week_2::highest_value_output(&transaction);
    assert_eq!(highest.unwrap().value, 50_000);
}

#[test]
fn recipient_filtering_returns_all_matches() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "alice"));
    transaction.add_output(output(50_000, "bob"));
    transaction.add_output(output(20_000, "alice"));

    let matches = rfb_labs_week_2::find_outputs_for_recipient(&transaction, "alice");
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].value, 10_000);
    assert_eq!(matches[1].value, 20_000);
}

#[test]
fn valid_coinbase_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_output(output(50_000, "miner"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn validation_fails_with_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "alice"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn validation_fails_with_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn validation_fails_with_zero_value_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_output(output(0, "alice")); // Not OpReturn
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::ZeroValueOutput)
    );
}

#[test]
fn zero_value_op_return_is_valid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "".into(),
        output_type: OutputType::OpReturn,
    });
    // Add a valid output so outputs don't exceed inputs (fee=10k, valid)
    transaction.add_output(output(5_000, "alice"));
    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn validation_fails_with_mixed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_output(output(10_000, "alice"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn validation_fails_with_multiple_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_input(InputKind::Coinbase {
        block_height: 101,
        reward: 50_000,
    });
    transaction.add_output(output(10_000, "alice"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn validation_fails_with_empty_txid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "".into(),
            vout: 0,
        },
        value: 10_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(10_000, "alice"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}
