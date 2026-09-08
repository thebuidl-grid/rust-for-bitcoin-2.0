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
fn highest_value_output_returns_the_largest() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(30_000, "bc1qsmall"));
    transaction.add_output(output(90_000, "bc1qbig"));

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 90_000);
    assert_eq!(highest.recipient, "bc1qbig");
}

#[test]
fn find_outputs_for_recipient_filters_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(50_000, "bc1qalice"));
    transaction.add_output(output(40_000, "bc1qbob"));
    transaction.add_output(output(20_000, "bc1qalice"));

    let alice_outputs = find_outputs_for_recipient(&transaction, "bc1qalice");
    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs.iter().map(|o| o.value).sum::<u64>(), 70_000);
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
}

#[test]
fn no_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn no_outputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn zero_value_non_op_return_output_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::ZeroValueOutput)
    );
}

#[test]
fn coinbase_mixed_with_regular_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));
    transaction.add_input(InputKind::Coinbase {
        block_height: 1,
        reward: 5_000_000_000,
    });
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 1,
        reward: 2_500_000_000,
    });
    transaction.add_input(InputKind::Coinbase {
        block_height: 1,
        reward: 2_500_000_000,
    });
    transaction.add_output(output(5_000_000_000, "bc1qminer"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::MultipleCoinbaseInputs)
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

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}
