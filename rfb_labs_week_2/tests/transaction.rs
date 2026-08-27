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

fn coinbase(reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: 1,
        reward,
    }
}

// These tests are ignored so the starter repository builds before students
// implement the TODOs. Remove the ignore attribute one at a time while working.

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
fn valid_coinbase_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase(50_000));
    transaction.add_output(output(49_500, "bc1qsender"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.fee(), Ok(500));
}

#[test]
fn no_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(90_000, "bc1qreceiver"));
    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn no_outputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(90_000));
    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn zero_value_non_op_return_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(90_000));
    transaction.add_output(output(0, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn multiple_coinbase_inputs_are_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase(40_000));
    transaction.add_input(coinbase(50_000));
    transaction.add_output(output(80_000, "bc1qsender"));
    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn coinbase_mixed_with_regular_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase(60_000));
    transaction.add_input(regular_input(20_000));
    transaction.add_output(output(70_000, "bc1qsender"));
    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn empty_regular_txid_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    let bad = InputKind::Regular {
        previous_output: OutPoint {
            txid: String::new(),
            vout: 0,
        },
        value: 100_000,
        sequence: u32::MAX,
    };
    transaction.add_input(bad);
    transaction.add_output(output(90_000, "bc1qreceiver"));
    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}

#[test]
fn fee_reports_outputs_exceeding_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));
    assert_eq!(
        transaction.fee(),
        Err(TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn highest_value_output_borrows_the_largest() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(28_000, "bc1qsender"));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 90_000);
    assert_eq!(highest.recipient, "bc1qreceiver");
}

#[test]
fn find_outputs_filters_by_recipient() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(1_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    let matches = find_outputs_for_recipient(&transaction, "bc1qreceiver");
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().all(|o| o.recipient == "bc1qreceiver"));
}
