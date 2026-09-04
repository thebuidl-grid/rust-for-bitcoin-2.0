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

fn coinbase_input(reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: 840_000,
        reward,
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
fn transaction_requires_an_input() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(1, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn transaction_requires_an_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn regular_outputs_cannot_have_zero_value() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::ZeroValueOutput)
    );
}

#[test]
fn op_return_output_may_have_zero_value() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "metadata".into(),
        output_type: OutputType::OpReturn,
    });

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn coinbase_cannot_be_mixed_with_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn transaction_cannot_have_multiple_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000));
    transaction.add_input(coinbase_input(50_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn regular_input_requires_a_txid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: String::new(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(40_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000));
    transaction.add_output(output(50_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn highest_value_output_borrows_the_largest_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(20_000, "bc1qfirst"));
    transaction.add_output(output(50_000, "bc1qhighest"));
    transaction.add_output(output(30_000, "bc1qthird"));

    let highest = highest_value_output(&transaction).expect("transaction has outputs");

    assert_eq!(highest.value, 50_000);
    assert_eq!(highest.recipient, "bc1qhighest");
}

#[test]
fn recipient_filter_returns_borrowed_matching_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(20_000, "bc1qreceiver"));
    transaction.add_output(output(30_000, "bc1qsender"));
    transaction.add_output(output(40_000, "bc1qreceiver"));

    let matching = find_outputs_for_recipient(&transaction, "bc1qreceiver");

    assert_eq!(matching.len(), 2);
    assert_eq!(matching[0].value, 20_000);
    assert_eq!(matching[1].value, 40_000);
}
