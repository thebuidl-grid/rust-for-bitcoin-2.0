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

fn coinbase_input(reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: 800_000,
        reward,
    }
}

#[test]
fn totals_are_calculated_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(70_000));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    assert_eq!(transaction.total_input_value(), 120_000);
    assert_eq!(transaction.total_output_value(), 118_000);
}

#[test]
fn fee_is_calculated_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(118_000, "bc1qreceiver"));

    assert_eq!(transaction.fee(), Ok(2_000));
}

#[test]
fn highest_value_output_finds_the_largest() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(30_000, "bc1qa"));
    transaction.add_output(output(90_000, "bc1qb"));

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 90_000);
    assert_eq!(highest.recipient, "bc1qb");
}

#[test]
fn find_outputs_for_recipient_returns_matches_only() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(30_000, "bc1qa"));
    transaction.add_output(output(90_000, "bc1qb"));
    transaction.add_output(output(10_000, "bc1qa"));

    let matches = find_outputs_for_recipient(&transaction, "bc1qa");
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().all(|o| o.recipient == "bc1qa"));
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_output(output(625_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn no_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn no_outputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn zero_value_non_op_return_output_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn zero_value_op_return_output_is_allowed() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: String::new(),
        output_type: OutputType::OpReturn,
    });
    transaction.add_output(output(500, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn mixed_coinbase_and_regular_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(500_000));
    transaction.add_input(regular_input(1_000));
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(500_000));
    transaction.add_input(coinbase_input(500_000));
    transaction.add_output(output(500_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn empty_txid_on_regular_input_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: String::new(),
            vout: 0,
        },
        value: 1_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(500, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}
