use rfb_labs_week_2::state::{TrackedTransaction, TransactionState};
use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TransactionError, TxOutput};

fn valid_transaction() -> Transaction {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value: 120_000,
        sequence: u32::MAX,
    });
    transaction.add_output(TxOutput {
        value: 118_000,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });
    transaction
}

#[test]
fn new_tracked_transaction_starts_as_created() {
    let tracked = TrackedTransaction::new(valid_transaction());
    assert_eq!(tracked.state(), TransactionState::Created);
}

#[test]
fn happy_path_walks_through_every_state() {
    let mut tracked = TrackedTransaction::new(valid_transaction());

    assert!(tracked.mark_validated().is_ok());
    assert_eq!(tracked.state(), TransactionState::Validated);

    assert!(tracked.mark_signed().is_ok());
    assert_eq!(tracked.state(), TransactionState::Signed);

    assert!(tracked.mark_broadcast().is_ok());
    assert_eq!(tracked.state(), TransactionState::Broadcast);

    assert!(tracked.mark_confirmed().is_ok());
    assert_eq!(tracked.state(), TransactionState::Confirmed);
}

#[test]
fn cannot_skip_straight_to_signed_from_created() {
    let mut tracked = TrackedTransaction::new(valid_transaction());

    let result = tracked.mark_signed();

    assert_eq!(
        result,
        Err(TransactionError::InvalidStateTransition {
            from: TransactionState::Created,
            to: TransactionState::Signed,
        })
    );
    // state must be unchanged after a rejected transition
    assert_eq!(tracked.state(), TransactionState::Created);
}

#[test]
fn cannot_skip_straight_to_broadcast_from_created() {
    let mut tracked = TrackedTransaction::new(valid_transaction());

    let result = tracked.mark_broadcast();

    assert_eq!(
        result,
        Err(TransactionError::InvalidStateTransition {
            from: TransactionState::Created,
            to: TransactionState::Broadcast,
        })
    );
}

#[test]
fn confirmed_is_a_terminal_state() {
    let mut tracked = TrackedTransaction::new(valid_transaction());
    tracked.mark_validated().unwrap();
    tracked.mark_signed().unwrap();
    tracked.mark_broadcast().unwrap();
    tracked.mark_confirmed().unwrap();

    // nothing should be able to move out of Confirmed
    let result = tracked.mark_rejected();

    assert_eq!(
        result,
        Err(TransactionError::InvalidStateTransition {
            from: TransactionState::Confirmed,
            to: TransactionState::Rejected,
        })
    );
}

#[test]
fn rejected_is_reachable_from_signed() {
    let mut tracked = TrackedTransaction::new(valid_transaction());
    tracked.mark_validated().unwrap();
    tracked.mark_signed().unwrap();

    assert!(tracked.mark_rejected().is_ok());
    assert_eq!(tracked.state(), TransactionState::Rejected);
}

#[test]
fn rejected_is_a_terminal_state() {
    let mut tracked = TrackedTransaction::new(valid_transaction());
    tracked.mark_rejected().unwrap();

    let result = tracked.mark_validated();

    assert_eq!(
        result,
        Err(TransactionError::InvalidStateTransition {
            from: TransactionState::Rejected,
            to: TransactionState::Validated,
        })
    );
}

#[test]
fn mark_validated_fails_if_underlying_transaction_is_invalid() {
    // no outputs at all -> the underlying `validate()` call should fail
    // before the state even has a chance to transition
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value: 1_000,
        sequence: u32::MAX,
    });
    let mut tracked = TrackedTransaction::new(transaction);

    let result = tracked.mark_validated();

    assert_eq!(result, Err(TransactionError::NoOutputs));
    // state should remain Created since validation never succeeded
    assert_eq!(tracked.state(), TransactionState::Created);
}
