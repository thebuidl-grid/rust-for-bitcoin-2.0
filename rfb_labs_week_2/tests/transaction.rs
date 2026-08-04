use rfb_labs_week_2::{
    find_outputs_for_recipient, highest_value_output, InputKind, OutPoint, OutputType, Transaction,
    TxOutput,
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
fn no_inputs_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn no_outputs_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn zero_value_non_op_return_output_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::ZeroValueOutput)
    );
}

#[test]
fn op_return_zero_value_is_allowed() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "data".to_string(),
        output_type: OutputType::OpReturn,
    });

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn mixed_coinbase_and_regular_inputs_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn empty_regular_txid_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}

#[test]
fn coinbase_only_transaction_is_valid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn total_input_value_sums_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(70_000));
    transaction.add_input(regular_input(50_000));

    assert_eq!(transaction.total_input_value(), 120_000);
}

#[test]
fn total_input_value_sums_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100,
        reward: 50_000,
    });

    assert_eq!(transaction.total_input_value(), 50_000);
}

#[test]
fn total_output_value_sums_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    assert_eq!(transaction.total_output_value(), 118_000);
}

#[test]
fn fee_calculates_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    assert_eq!(transaction.fee(), Ok(2_000));
}

#[test]
fn fee_returns_error_when_outputs_exceed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));

    assert_eq!(
        transaction.fee(),
        Err(rfb_labs_week_2::TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn highest_value_output_returns_largest() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    assert_eq!(
        highest_value_output(&transaction),
        Some(&output(90_000, "bc1qreceiver"))
    );
}

#[test]
fn highest_value_output_returns_none_when_empty() {
    let transaction = Transaction::new(2, 0);
    assert_eq!(highest_value_output(&transaction), None);
}

#[test]
fn find_outputs_for_recipient_filters_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));
    transaction.add_output(output(0, "data"));

    let results = find_outputs_for_recipient(&transaction, "bc1qreceiver");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 90_000);
}

#[test]
fn find_outputs_for_recipient_returns_empty_when_none_match() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    let results = find_outputs_for_recipient(&transaction, "bc1qsender");
    assert!(results.is_empty());
}

#[test]
fn display_outpoint_formats_correctly() {
    let outpoint = OutPoint {
        txid: "abcd".into(),
        vout: 1,
    };
    assert_eq!(format!("{outpoint}"), "abcd:1");
}

#[test]
fn display_transaction_summary() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    let display = format!("{transaction}");
    assert!(display.contains("Transaction v2"));
    assert!(display.contains("Inputs: 1"));
    assert!(display.contains("Outputs: 2"));
    assert!(display.contains("Fee: 2000 sats"));
}
