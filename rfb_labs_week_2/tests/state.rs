//! Part 10 (optional): the transaction lifecycle rejects invalid transitions.

use rfb_labs_week_2::{InvalidTransition, TransactionLifecycle, TransactionState};

#[test]
fn a_new_lifecycle_starts_as_created() {
    assert_eq!(
        TransactionLifecycle::new().state(),
        TransactionState::Created
    );
}

#[test]
fn the_happy_path_runs_all_the_way_to_confirmed() {
    let mut lifecycle = TransactionLifecycle::new();

    for state in [
        TransactionState::Validated,
        TransactionState::Signed,
        TransactionState::Broadcast,
        TransactionState::Confirmed,
    ] {
        assert_eq!(lifecycle.advance_to(state), Ok(state));
    }

    assert_eq!(lifecycle.state(), TransactionState::Confirmed);
}

#[test]
fn stages_cannot_be_skipped() {
    let mut lifecycle = TransactionLifecycle::new();

    assert_eq!(
        lifecycle.advance_to(TransactionState::Broadcast),
        Err(InvalidTransition {
            from: TransactionState::Created,
            to: TransactionState::Broadcast,
        })
    );
    // A refused transition leaves the state untouched.
    assert_eq!(lifecycle.state(), TransactionState::Created);
}

#[test]
fn any_unfinished_stage_may_be_rejected() {
    for state in [
        TransactionState::Created,
        TransactionState::Validated,
        TransactionState::Signed,
        TransactionState::Broadcast,
    ] {
        assert!(state.can_transition_to(TransactionState::Rejected));
    }
}

#[test]
fn terminal_states_accept_nothing_further() {
    let mut confirmed = TransactionLifecycle::new();
    for state in [
        TransactionState::Validated,
        TransactionState::Signed,
        TransactionState::Broadcast,
        TransactionState::Confirmed,
    ] {
        confirmed.advance_to(state).unwrap();
    }

    assert!(confirmed.state().is_final());
    assert!(confirmed.advance_to(TransactionState::Rejected).is_err());
    assert!(confirmed.advance_to(TransactionState::Broadcast).is_err());
}

#[test]
fn a_rejected_transaction_cannot_be_revived() {
    let mut lifecycle = TransactionLifecycle::new();
    lifecycle.advance_to(TransactionState::Rejected).unwrap();

    assert!(lifecycle.state().is_final());
    assert!(lifecycle.advance_to(TransactionState::Validated).is_err());
}

#[test]
fn an_invalid_transition_explains_itself() {
    let error = InvalidTransition {
        from: TransactionState::Created,
        to: TransactionState::Broadcast,
    };

    assert_eq!(
        error.to_string(),
        "a created transaction cannot move to broadcast"
    );
}
