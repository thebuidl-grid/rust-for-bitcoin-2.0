use rfb_labs_week_2::{
    find_outputs_for_recipient, highest_value_output, InputKind, OutPoint, OutputType, Transaction,
    TransactionError, TransactionState, TxOutput,
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
        block_height: 100,
        reward,
    }
}

fn regular_input_with_empty_txid(value: u64) -> InputKind {
    InputKind::Regular {
        previous_output: OutPoint {
            txid: String::new(),
            vout: 0,
        },
        value,
        sequence: u32::MAX,
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
fn valid_state_transition_succeeds() {
    let mut state = TransactionState::Created;

    assert_eq!(state.transition_to(TransactionState::Validated), Ok(()));
    assert_eq!(state, TransactionState::Validated);
}

#[test]
fn invalid_state_transition_is_rejected() {
    let mut state = TransactionState::Created;

    assert_eq!(
        state.transition_to(TransactionState::Signed),
        Err(TransactionError::InvalidStateTransition {
            from: TransactionState::Created,
            to: TransactionState::Signed,
        })
    );
    assert_eq!(state, TransactionState::Created);
}

#[test]
fn finds_highest_value_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qsmall"));
    transaction.add_output(output(90_000, "bc1qbig"));
    transaction.add_output(output(50_000, "bc1qmedium"));

    assert_eq!(
        highest_value_output(&transaction),
        Some(&output(90_000, "bc1qbig"))
    );
}

#[test]
fn finds_outputs_for_recipient() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qalice"));
    transaction.add_output(output(20_000, "bc1qbob"));
    transaction.add_output(output(30_000, "bc1qalice"));

    let alice_outputs = find_outputs_for_recipient(&transaction, "bc1qalice");

    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs.iter().map(|o| o.value).sum::<u64>(), 40_000);
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_output(output(625_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.fee(), Ok(0));
}

#[test]
fn rejects_transaction_with_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn rejects_transaction_with_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn rejects_non_op_return_zero_value_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn rejects_coinbase_mixed_with_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_input(regular_input(10_000));
    transaction.add_output(output(10_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn rejects_multiple_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_input(coinbase_input(625_000_000));
    transaction.add_output(output(1_250_000_000, "bc1qminer"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn rejects_regular_input_with_empty_txid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input_with_empty_txid(10_000));
    transaction.add_output(output(10_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}
