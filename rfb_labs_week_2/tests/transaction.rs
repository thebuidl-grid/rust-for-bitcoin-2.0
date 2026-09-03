use rfb_labs_week_2::{
    find_outputs_for_recipient, highest_value_output, BitcoinValue, InputKind, OutPoint,
    OutputType, Transaction, TransactionError, TxOutput,
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
        Err(TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
    assert_eq!(
        transaction.fee(),
        Err(TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn empty_transaction_has_no_inputs() {
    let transaction = Transaction::new(2, 0);
    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn transaction_with_only_inputs_has_no_outputs() {
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
        recipient: "OP_RETURN payload".into(),
        output_type: OutputType::OpReturn,
    });
    transaction.add_output(output(500, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Ok(()));
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

#[test]
fn coinbase_cannot_be_mixed_with_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));
    transaction.add_input(InputKind::Coinbase {
        block_height: 800_000,
        reward: 625_000_000,
    });
    transaction.add_output(output(500, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_are_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 800_000,
        reward: 300_000_000,
    });
    transaction.add_input(InputKind::Coinbase {
        block_height: 800_000,
        reward: 300_000_000,
    });
    transaction.add_output(output(500_000_000, "bc1qminer"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 800_000,
        reward: 625_000_000,
    });
    transaction.add_output(output(625_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 625_000_000);
    assert_eq!(transaction.fee(), Ok(0));
}

#[test]
fn highest_value_output_returns_the_largest_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    let highest = highest_value_output(&transaction).expect("an output should exist");
    assert_eq!(highest.value, 90_000);
    assert_eq!(highest.recipient, "bc1qreceiver");
}

#[test]
fn highest_value_output_is_none_for_empty_outputs() {
    let transaction = Transaction::new(2, 0);
    assert!(highest_value_output(&transaction).is_none());
}

#[test]
fn find_outputs_for_recipient_returns_all_matches() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(200_000));
    transaction.add_output(output(50_000, "bc1qreceiver"));
    transaction.add_output(output(60_000, "bc1qreceiver"));
    transaction.add_output(output(70_000, "bc1qsender"));

    let matches = find_outputs_for_recipient(&transaction, "bc1qreceiver");
    assert_eq!(matches.len(), 2);
    assert_eq!(matches.iter().map(|o| o.value).sum::<u64>(), 110_000);

    let none = find_outputs_for_recipient(&transaction, "bc1qnobody");
    assert!(none.is_empty());
}

#[test]
fn bitcoin_value_trait_reports_value_in_btc() {
    let regular = regular_input(150_000_000);
    assert_eq!(regular.value(), 150_000_000);
    assert!((regular.value_in_btc() - 1.5).abs() < f64::EPSILON);

    let coinbase = InputKind::Coinbase {
        block_height: 1,
        reward: 100_000_000,
    };
    assert_eq!(coinbase.value(), 100_000_000);
}

#[test]
fn display_impls_produce_readable_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    let rendered = transaction.to_string();
    assert!(rendered.contains("1 input(s)"));
    assert!(rendered.contains("2 output(s)"));
    assert!(rendered.contains("fee=2000 sats"));
    assert!(rendered.contains("total_in=120000 sats"));
}
