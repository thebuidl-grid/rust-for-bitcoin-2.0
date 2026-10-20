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

fn coinbase_input(reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: 100,
        reward,
    }
}

fn op_return_output() -> TxOutput {
    TxOutput {
        value: 0,
        recipient: "op_return".into(),
        output_type: OutputType::OpReturn,
    }
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_output(output(625_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 625_000_000);
    assert_eq!(transaction.fee(), Ok(0));
}

#[test]
fn highest_value_output_returns_the_largest_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(200_000));
    transaction.add_output(output(50_000, "bc1qsmall"));
    transaction.add_output(output(120_000, "bc1qbig"));
    transaction.add_output(output(30_000, "bc1qtiny"));

    let highest = rfb_labs_week_2::highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 120_000);
    assert_eq!(highest.recipient, "bc1qbig");
}

#[test]
fn highest_value_output_is_none_without_outputs() {
    let transaction = Transaction::new(2, 0);
    assert!(rfb_labs_week_2::highest_value_output(&transaction).is_none());
}

#[test]
fn find_outputs_for_recipient_returns_only_matches() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(200_000));
    transaction.add_output(output(50_000, "bc1qalice"));
    transaction.add_output(output(60_000, "bc1qbob"));
    transaction.add_output(output(70_000, "bc1qalice"));

    let alice_outputs = rfb_labs_week_2::find_outputs_for_recipient(&transaction, "bc1qalice");
    assert_eq!(alice_outputs.len(), 2);
    assert!(alice_outputs.iter().all(|out| out.recipient == "bc1qalice"));
}

#[test]
fn rejects_transaction_with_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn rejects_transaction_with_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn rejects_zero_value_non_op_return_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::ZeroValueOutput)
    );
}

#[test]
fn allows_zero_value_op_return_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));
    transaction.add_output(op_return_output());
    transaction.add_output(output(500, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn rejects_coinbase_mixed_with_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn rejects_multiple_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn rejects_regular_input_with_empty_txid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: String::new(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}
