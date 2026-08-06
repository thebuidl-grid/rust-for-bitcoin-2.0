use assigment2::{
    Address, CoinSelectionStrategy, OutPoint, TransactionStatus, TxOutput, Validate, Wallet,
};

#[test]
fn end_to_end_payment_spends_utxos_and_confirms() {
    let mut wallet = Wallet::new(Address::from("change"));
    wallet.fund(
        OutPoint::new("fund-a", 0),
        TxOutput {
            value: 5_000,
            address: Address::from("mine"),
        },
    );
    wallet.fund(
        OutPoint::new("fund-b", 0),
        TxOutput {
            value: 1_500,
            address: Address::from("mine"),
        },
    );

    let mut tx = wallet
        .create_transaction(
            vec![TxOutput {
                value: 4_000,
                address: Address::from("merchant"),
            }],
            100,
            CoinSelectionStrategy::LargestFirst,
        )
        .expect("payment should succeed with sufficient funds");

    tx.validate()
        .expect("wallet-built transactions must already be valid");
    assert_eq!(tx.status(), TransactionStatus::Draft);

    tx.advance_status(TransactionStatus::Signed).unwrap();
    tx.advance_status(TransactionStatus::Broadcast).unwrap();
    tx.advance_status(TransactionStatus::Confirmed { height: 100 })
        .unwrap();
    assert_eq!(tx.status(), TransactionStatus::Confirmed { height: 100 });

    assert_eq!(wallet.balance(), 5_000 + 1_500 - 4_000 - 100);
}

#[test]
fn payment_beyond_balance_is_rejected_without_mutating_wallet() {
    let mut wallet = Wallet::new(Address::from("change"));
    wallet.fund(
        OutPoint::new("fund-a", 0),
        TxOutput {
            value: 500,
            address: Address::from("mine"),
        },
    );

    let err = wallet
        .create_transaction(
            vec![TxOutput {
                value: 10_000,
                address: Address::from("merchant"),
            }],
            0,
            CoinSelectionStrategy::SmallestFirst,
        )
        .unwrap_err();

    assert!(err.to_string().contains("insufficient funds"));
    assert_eq!(wallet.balance(), 500);
}
