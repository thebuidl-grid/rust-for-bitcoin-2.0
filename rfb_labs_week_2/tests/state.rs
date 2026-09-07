use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TransactionError, TxOutput};

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

fn valid_transaction() -> Transaction {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));
    transaction
}

#[test]
fn happy_path_reaches_confirmed() {
    let mut transaction = valid_transaction();
    transaction.mark_validated().unwrap();
    transaction.sign().unwrap();
    transaction.broadcast().unwrap();
    transaction.confirm().unwrap();
}

#[test]
fn sign_before_validate_is_an_error() {
    let mut transaction = valid_transaction();
    assert_eq!(
        transaction.sign(),
        Err(TransactionError::InvalidStateTransition {
            from: rfb_labs_week_2::TransactionState::Created,
            to: rfb_labs_week_2::TransactionState::Signed,
        })
    );
}

#[test]
fn broadcast_before_sign_is_an_error() {
    let mut transaction = valid_transaction();
    transaction.mark_validated().unwrap();
    assert!(transaction.broadcast().is_err());
}

#[test]
fn confirm_before_broadcast_is_an_error() {
    let mut transaction = valid_transaction();
    transaction.mark_validated().unwrap();
    transaction.sign().unwrap();
    assert!(transaction.confirm().is_err());
}

#[test]
fn rejected_transaction_cannot_be_signed() {
    let mut transaction = valid_transaction();
    transaction.mark_validated().unwrap();
    transaction.reject().unwrap();
    assert!(transaction.sign().is_err());
}

#[test]
fn confirmed_transaction_cannot_be_rejected() {
    let mut transaction = valid_transaction();
    transaction.mark_validated().unwrap();
    transaction.sign().unwrap();
    transaction.broadcast().unwrap();
    transaction.confirm().unwrap();
    assert!(transaction.reject().is_err());
}
