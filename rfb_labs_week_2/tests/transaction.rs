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

fn regular_input_with_txid(txid: &str, value: u64) -> InputKind {
    InputKind::Regular {
        previous_output: OutPoint {
            txid: txid.into(),
            vout: 0,
        },
        value,
        sequence: u32::MAX,
    }
}

fn coinbase_input(block_height: u32, reward: u64) -> InputKind {
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
        Err(rfb_labs_week_2::TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn highest_value_output_returns_max_output() {
    let mut tx = Transaction::new(2, 0);
    tx.add_input(regular_input_with_txid(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        120_000,
    ));
    tx.add_output(output(28_000, "bc1qsender"));
    tx.add_output(output(90_000, "bc1qreceiver"));

    assert_eq!(highest_value_output(&tx), Some(&tx.outputs[1]));
    assert_eq!(highest_value_output(&tx).unwrap().value, 90_000);

    let empty = Transaction::new(2, 0);
    assert_eq!(highest_value_output(&empty), None);
}

#[test]
fn find_outputs_for_recipient_filters_by_address() {
    let mut tx = Transaction::new(2, 0);
    tx.add_input(regular_input_with_txid(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        120_000,
    ));
    tx.add_output(output(90_000, "bc1qreceiver"));
    tx.add_output(output(28_000, "bc1qsender"));

    let sender = find_outputs_for_recipient(&tx, "bc1qsender");
    assert_eq!(sender.len(), 1);
    assert_eq!(sender[0].value, 28_000);

    assert!(find_outputs_for_recipient(&tx, "bc1qnobody").is_empty());
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut tx = Transaction::new(2, 0);
    tx.add_input(coinbase_input(100, 50_000));
    tx.add_output(output(45_000, "bc1qreceiver"));

    assert_eq!(tx.validate(), Ok(()));
    assert_eq!(tx.fee(), Ok(5_000));
}

#[test]
fn no_inputs_is_rejected() {
    let mut tx = Transaction::new(2, 0);
    tx.add_output(output(10_000, "bc1qreceiver"));

    assert_eq!(tx.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn no_outputs_is_rejected() {
    let mut tx = Transaction::new(2, 0);
    tx.add_input(regular_input_with_txid(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        10_000,
    ));

    assert_eq!(tx.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn zero_value_output_is_rejected() {
    let mut tx = Transaction::new(2, 0);
    tx.add_input(regular_input_with_txid(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        10_000,
    ));
    tx.add_output(output(0, "bc1qreceiver"));

    assert_eq!(tx.validate(), Err(TransactionError::ZeroValueOutput));
}

#[test]
fn empty_txid_is_rejected() {
    let mut tx = Transaction::new(2, 0);
    tx.add_input(regular_input_with_txid("", 10_000));
    tx.add_output(output(5_000, "bc1qreceiver"));

    assert_eq!(tx.validate(), Err(TransactionError::InvalidTxid));
}

#[test]
fn mixed_coinbase_and_regular_inputs_is_rejected() {
    let mut tx = Transaction::new(2, 0);
    tx.add_input(regular_input_with_txid(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        100_000,
    ));
    tx.add_input(coinbase_input(100, 50_000));
    tx.add_output(output(100_000, "bc1qreceiver"));

    assert_eq!(
        tx.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_is_rejected() {
    let mut tx = Transaction::new(2, 0);
    tx.add_input(coinbase_input(100, 50_000));
    tx.add_input(coinbase_input(101, 50_000));
    tx.add_output(output(90_000, "bc1qreceiver"));

    assert_eq!(tx.validate(), Err(TransactionError::MultipleCoinbaseInputs));
}
