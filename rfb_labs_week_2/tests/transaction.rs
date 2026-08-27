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

fn coinbase_input(reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: 800_000,
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
fn highest_value_output_returns_the_largest() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(30_000, "bc1qsmall"));
    transaction.add_output(output(90_000, "bc1qbig"));

    let highest = highest_value_output(&transaction).expect("an output should exist");
    assert_eq!(highest.value, 90_000);
    assert_eq!(highest.recipient, "bc1qbig");
}

#[test]
fn find_outputs_for_recipient_returns_matches() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(50_000, "bc1qreceiver"));
    transaction.add_output(output(20_000, "bc1qreceiver"));
    transaction.add_output(output(40_000, "bc1qsender"));

    let matches = find_outputs_for_recipient(&transaction, "bc1qreceiver");
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().all(|out| out.recipient == "bc1qreceiver"));
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_output(output(625_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn transaction_with_no_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn transaction_with_no_outputs_is_rejected() {
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
        recipient: "OP_RETURN".into(),
        output_type: OutputType::OpReturn,
    });
    transaction.add_output(output(1_000, "bc1qsender"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn mixed_coinbase_and_regular_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(1_000));
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
    transaction.add_input(coinbase_input(1_000));
    transaction.add_input(coinbase_input(1_000));
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn empty_regular_txid_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input_with_txid(1_000, ""));
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}
