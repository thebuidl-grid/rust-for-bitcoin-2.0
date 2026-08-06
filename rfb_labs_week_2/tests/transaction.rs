use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn regular_input(value: u64) -> InputKind {
    InputKind::Regular {
        previous_output: OutPoint {
            txid: "a_dummy_tx_id".into(),
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
        output_type: OutputType::P2pkh,
    }
}

// These tests are ignored so the starter repository builds before students
// implement the TODOs. Remove `#[ignore]` from one test at a time while working.

#[test]
// #[ignore = "enable after completing Parts 3 and 5"]
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
// #[ignore = "enable after completing Part 5"]
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
fn fee() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(70_000));
    transaction.add_output(output(50_000, "bc1qreceiver"));
    transaction.add_output(output(18_500, "bc1qsender"));

    assert_eq!(transaction.fee(), Ok(1500));
}

#[test]
fn highest_value_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(70_000));
    transaction.add_output(output(18_500, "bc1qsender"));
    transaction.add_output(output(50_000, "bc1qreceiver"));

    let highest = rfb_labs_week_2::highest_value_output(&transaction);
    assert_eq!(highest, Some(&output(50_000, "bc1qreceiver")));
}

#[test]
fn find_outputs_for_recipient() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(70_000));
    transaction.add_output(output(50_000, "bc1qreceiver"));
    transaction.add_output(output(18_500, "bc1qsender"));

    let results = rfb_labs_week_2::find_outputs_for_recipient(&transaction, "bc1qreceiver");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 50_000);
}

#[test]
fn validation_coinbase_transaction() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 1,
        reward: 50_000_000,
    });
    transaction.add_output(output(50_000_000, "bc1qsender"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn validation_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn validation_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(70_000));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn validation_mixed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 1,
        reward: 50_000_000,
    });
    transaction.add_input(regular_input(70_000));
    transaction.add_output(output(75_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn validation_empty_txid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}
