use rfb_labs_week_2::{
    InputKind, Lifecycle, OutPoint, OutputType, RejectionReason, Transaction, TransactionError,
    TxOutput,
};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn funded_transaction() -> Transaction {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: TXID.into(),
            vout: 0,
        },
        value: 120_000,
        sequence: u32::MAX,
    });
    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });
    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    });
    transaction
}

#[test]
fn a_valid_transaction_walks_the_whole_lifecycle() {
    let confirmed = Lifecycle::new(funded_transaction())
        .validate()
        .expect("transaction is valid")
        .sign("3044-placeholder-signature")
        .broadcast()
        .confirm(850_123);

    assert_eq!(confirmed.block_height(), 850_123);
    assert_eq!(confirmed.transaction().fee(), Ok(2_000));
}

#[test]
fn validation_failure_moves_the_transaction_to_rejected() {
    // No inputs and no outputs.
    let rejected = Lifecycle::new(Transaction::new(2, 0))
        .validate()
        .expect_err("an empty transaction cannot validate");

    assert_eq!(
        rejected.reason(),
        &RejectionReason::ValidationFailed(TransactionError::NoInputs)
    );

    // The transaction is still readable after rejection.
    assert_eq!(rejected.transaction().version, 2);
}

#[test]
fn the_network_can_reject_a_broadcast_transaction() {
    let rejected = Lifecycle::new(funded_transaction())
        .validate()
        .expect("transaction is valid")
        .sign("signature")
        .broadcast()
        .reject("min relay fee not met");

    assert_eq!(
        rejected.reason(),
        &RejectionReason::RefusedByNetwork("min relay fee not met".into())
    );
}

#[test]
fn the_signature_is_carried_by_the_signed_state() {
    let signed = Lifecycle::new(funded_transaction())
        .validate()
        .expect("transaction is valid")
        .sign("3044-placeholder-signature");

    assert_eq!(signed.state().signature, "3044-placeholder-signature");
}

#[test]
fn display_reports_the_current_state() {
    let created = Lifecycle::new(funded_transaction());
    assert!(created.to_string().starts_with("[Created]"));

    let validated = created.validate().expect("transaction is valid");
    assert!(validated.to_string().starts_with("[Validated]"));

    let confirmed = validated.sign("signature").broadcast().confirm(1);
    assert!(confirmed.to_string().starts_with("[Confirmed]"));
}

#[test]
fn rejection_reasons_have_readable_messages() {
    let validation = RejectionReason::ValidationFailed(TransactionError::NoOutputs);
    assert!(validation.to_string().contains("validation failed"));

    let network = RejectionReason::RefusedByNetwork("mempool full".into());
    assert!(network.to_string().contains("mempool full"));
}
