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
fn coinbase_input(height: u32, reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: height,
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
// #[ignore = "enable after completing Parts 3 and 5"]
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
// #[ignore = "enable after completing Part 5"]
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
fn zero_value_non_opreturn_output_fails() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(0, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn mixing_coinbase_and_regular_inputs_fails() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_input(coinbase_input(100, 50_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn highest_value_output_selection() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qreceiver"));
    transaction.add_output(output(50_000, "bc1qsender"));

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 50_000);
}

#[test]
fn recipient_output_filtering() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "alice"));
    transaction.add_output(output(20_000, "bob"));
    transaction.add_output(output(30_000, "alice"));

    let alice_outputs = find_outputs_for_recipient(&transaction, "alice");
    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs.iter().map(|o| o.value).sum::<u64>(), 40_000);
}
