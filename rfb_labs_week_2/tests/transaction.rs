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

fn regular_input_with_txid(value: u64, txid: &str) -> InputKind {
    InputKind::Regular {
        previous_output: OutPoint {
            txid: txid.into(),
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
        Err(TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn highest_value_output_returns_the_largest_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(30_000, "bc1qsmall"));
    transaction.add_output(output(90_000, "bc1qbig"));

    let highest = highest_value_output(&transaction).expect("an output should exist");
    assert_eq!(highest.value, 90_000);
    assert_eq!(highest.recipient, "bc1qbig");
}

#[test]
fn highest_value_output_is_none_without_outputs() {
    let transaction = Transaction::new(2, 0);
    assert!(highest_value_output(&transaction).is_none());
}

#[test]
fn find_outputs_for_recipient_filters_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(50_000, "bc1qreceiver"));
    transaction.add_output(output(20_000, "bc1qsender"));
    transaction.add_output(output(40_000, "bc1qreceiver"));

    let matches = find_outputs_for_recipient(&transaction, "bc1qreceiver");
    assert_eq!(matches.len(), 2);
    assert_eq!(matches.iter().map(|out| out.value).sum::<u64>(), 90_000);

    let none = find_outputs_for_recipient(&transaction, "bc1qunknown");
    assert!(none.is_empty());
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 840_000,
        reward: 312_500_000,
    });
    transaction.add_output(output(312_500_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 312_500_000);
    assert_eq!(transaction.fee(), Ok(0));
}

#[test]
fn rejects_transaction_with_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn rejects_transaction_with_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn rejects_zero_value_non_op_return_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn allows_zero_value_op_return_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_output(output(10_000, "bc1qreceiver"));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "OP_RETURN deadbeef".into(),
        output_type: OutputType::OpReturn,
    });

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn rejects_mixed_coinbase_and_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 840_000,
        reward: 100_000,
    });
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(100_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn rejects_multiple_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 840_000,
        reward: 100_000,
    });
    transaction.add_input(InputKind::Coinbase {
        block_height: 840_000,
        reward: 100_000,
    });
    transaction.add_output(output(200_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn rejects_empty_txid_on_regular_input() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input_with_txid(10_000, ""));
    transaction.add_output(output(10_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}

#[test]
fn transaction_display_shows_summary() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    let rendered = transaction.to_string();
    assert!(rendered.contains("1 input(s)"));
    assert!(rendered.contains("2 output(s)"));
    assert!(rendered.contains("fee 2000 sats"));
}

#[test]
fn transaction_display_reports_invalid_fee_without_panicking() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_output(output(20_000, "bc1qreceiver"));

    let rendered = transaction.to_string();
    assert!(rendered.contains("invalid"));
}
