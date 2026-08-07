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

fn coinbase_input(reward: u64, block_height: u32) -> InputKind {
    InputKind::Coinbase {
        block_height,
        reward,
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
        Err(TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn valid_coinbase_transaction() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000, 840_000));
    transaction.add_output(output(625_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 625_000_000);
    assert_eq!(transaction.fee(), Ok(0));
}

#[test]
fn highest_value_output_returns_largest() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(150_000));
    transaction.add_output(output(50_000, "bc1qalice"));
    transaction.add_output(output(90_000, "bc1qbob"));
    transaction.add_output(output(5_000, "bc1qcharlie"));

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 90_000);
    assert_eq!(highest.recipient, "bc1qbob");
}

#[test]
fn highest_value_output_returns_none_for_empty() {
    let transaction = Transaction::new(2, 0);
    assert!(highest_value_output(&transaction).is_none());
}

#[test]
fn find_outputs_for_recipient_filters_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(200_000));
    transaction.add_output(output(50_000, "bc1qalice"));
    transaction.add_output(output(30_000, "bc1qbob"));
    transaction.add_output(output(70_000, "bc1qalice"));
    transaction.add_output(output(40_000, "bc1qcharlie"));

    let alice_outputs = find_outputs_for_recipient(&transaction, "bc1qalice");
    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs[0].value, 50_000);
    assert_eq!(alice_outputs[1].value, 70_000);

    let bob_outputs = find_outputs_for_recipient(&transaction, "bc1qbob");
    assert_eq!(bob_outputs.len(), 1);
    assert_eq!(bob_outputs[0].value, 30_000);

    let nobody_outputs = find_outputs_for_recipient(&transaction, "bc1qnobody");
    assert_eq!(nobody_outputs.len(), 0);
}

#[test]
fn no_inputs_fails_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn no_outputs_fails_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn zero_value_non_opreturn_output_fails() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn zero_value_opreturn_is_allowed() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(50_000, "bc1qreceiver"));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "".into(),
        output_type: OutputType::OpReturn,
    });

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn mixed_coinbase_and_regular_inputs_fails() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000, 840_000));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(625_050_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_fails() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(312_500_000, 840_000));
    transaction.add_input(coinbase_input(312_500_000, 840_000));
    transaction.add_output(output(625_000_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn empty_txid_in_regular_input_fails() {
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

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}
