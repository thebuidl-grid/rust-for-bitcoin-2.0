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

fn op_return_output(recipient: &str) -> TxOutput {
    TxOutput {
        value: 0,
        recipient: recipient.into(),
        output_type: OutputType::OpReturn,
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
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000_000, 100));
    transaction.add_output(output(50_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 50_000_000);
    assert_eq!(transaction.total_output_value(), 50_000_000);
    assert_eq!(transaction.fee(), Ok(0));
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
fn empty_inputs_fails_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn empty_outputs_fails_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn zero_value_non_op_return_output_fails_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn zero_value_op_return_output_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(op_return_output("OP_RETURN data"));
    transaction.add_output(output(49_000, "bc1qsender"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn mixing_coinbase_and_regular_inputs_fails_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000, 1));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_fails_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000, 1));
    transaction.add_input(coinbase_input(50_000, 2));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn empty_txid_fails_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "  ".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(40_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}

#[test]
fn highest_value_output_finds_max() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(100_000));
    transaction.add_output(output(10_000, "alice"));
    transaction.add_output(output(60_000, "bob"));
    transaction.add_output(output(25_000, "charlie"));

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.recipient, "bob");
    assert_eq!(highest.value, 60_000);
}

#[test]
fn find_outputs_for_recipient_filters_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(100_000));
    transaction.add_output(output(10_000, "alice"));
    transaction.add_output(output(60_000, "bob"));
    transaction.add_output(output(25_000, "alice"));

    let alice_outputs = find_outputs_for_recipient(&transaction, "alice");
    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs.iter().map(|o| o.value).sum::<u64>(), 35_000);
}
