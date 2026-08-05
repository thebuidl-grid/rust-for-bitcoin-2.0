use rfb_labs_week_2::{
    find_outputs_for_recipient, highest_value_output, BitcoinValue, InputKind, OutPoint,
    OutputType, Transaction, TransactionError, TxOutput,
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

fn coinbase_input(reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: 100,
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
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(5_000_000_000));
    transaction.add_output(output(5_000_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.fee(), Ok(0));
}

#[test]
fn highest_value_output_finds_max() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(100_000));
    transaction.add_output(output(30_000, "alice"));
    transaction.add_output(output(60_000, "bob"));

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 60_000);
    assert_eq!(highest.recipient, "bob");
}

#[test]
fn find_outputs_for_recipient_filters_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(100_000));
    transaction.add_output(output(30_000, "alice"));
    transaction.add_output(output(40_000, "bob"));
    transaction.add_output(output(20_000, "alice"));

    let alice_outputs = find_outputs_for_recipient(&transaction, "alice");
    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs[0].value, 30_000);
    assert_eq!(alice_outputs[1].value, 20_000);
}

#[test]
fn validation_fails_with_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(50_000, "receiver"));
    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn validation_fails_with_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn validation_fails_with_zero_value_non_opreturn_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "receiver".into(),
        output_type: OutputType::P2wpkh,
    });
    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn op_return_zero_value_output_is_valid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(40_000, "receiver"));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "data".into(),
        output_type: OutputType::OpReturn,
    });
    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn validation_fails_when_mixing_coinbase_and_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(90_000, "receiver"));
    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn validation_fails_with_multiple_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000));
    transaction.add_input(coinbase_input(50_000));
    transaction.add_output(output(90_000, "receiver"));
    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn validation_fails_with_empty_regular_input_txid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(40_000, "receiver"));
    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}

#[test]
fn bitcoin_value_trait_converts_sats_to_btc() {
    let out = output(100_000_000, "receiver");
    assert_eq!(out.value(), 100_000_000);
    assert_eq!(out.value_in_btc(), 1.0);
}

#[test]
fn display_formatting_works_correctly() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(100_000));
    transaction.add_output(output(95_000, "receiver"));

    let display_str = format!("{transaction}");
    assert!(display_str.contains("Transaction v2"));
    assert!(display_str.contains("1 inputs (100000 sats)"));
    assert!(display_str.contains("1 outputs (95000 sats)"));
    assert!(display_str.contains("fee: 5000 sats"));
}
