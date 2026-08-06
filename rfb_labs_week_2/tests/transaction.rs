use rfb_labs_week_2::{
    find_outputs_for_recipient, highest_value_output, BitcoinValue, InputKind, OutPoint,
    OutputType, Transaction, TransactionError, TxOutput,
};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn regular_input(value: u64) -> InputKind {
    InputKind::Regular {
        previous_output: OutPoint {
            txid: TXID.into(),
            vout: 0,
        },
        value,
        sequence: u32::MAX,
    }
}

fn coinbase_input(reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: 850_000,
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

fn op_return(value: u64) -> TxOutput {
    TxOutput {
        value,
        recipient: "OP_RETURN".into(),
        output_type: OutputType::OpReturn,
    }
}

/// A funded transaction paying one recipient with change back to the sender.
fn funded_transaction() -> Transaction {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));
    transaction
}

// --- Happy paths -----------------------------------------------------------

#[test]
fn valid_regular_transaction_passes_validation() {
    let transaction = funded_transaction();

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 120_000);
    assert_eq!(transaction.total_output_value(), 118_000);
    assert_eq!(transaction.fee(), Ok(2_000));
}

#[test]
fn totals_cover_both_input_variants() {
    let mut regular = Transaction::new(2, 0);
    regular.add_input(regular_input(70_000));
    regular.add_input(regular_input(50_000));
    regular.add_output(output(100_000, "bc1qreceiver"));

    assert_eq!(regular.total_input_value(), 120_000);
    assert_eq!(regular.total_output_value(), 100_000);

    let mut coinbase = Transaction::new(2, 0);
    coinbase.add_input(coinbase_input(312_500_000));
    coinbase.add_output(output(312_500_000, "bc1qminer"));

    assert_eq!(coinbase.total_input_value(), 312_500_000);
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(312_500_000));
    transaction.add_output(output(312_500_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.fee(), Ok(0));
}

#[test]
fn highest_value_output_borrows_the_largest_output() {
    let transaction = funded_transaction();
    let largest = highest_value_output(&transaction).expect("transaction has outputs");

    assert_eq!(largest.value, 90_000);
    assert_eq!(largest.recipient, "bc1qreceiver");

    // The borrow points into the transaction rather than a copy.
    assert!(std::ptr::eq(largest, &transaction.outputs[0]));

    assert!(highest_value_output(&Transaction::new(2, 0)).is_none());
}

#[test]
fn recipient_filtering_returns_every_matching_output() {
    let mut transaction = funded_transaction();
    transaction.add_output(output(1_000, "bc1qreceiver"));

    let matches = find_outputs_for_recipient(&transaction, "bc1qreceiver");
    assert_eq!(matches.len(), 2);
    assert_eq!(
        matches.iter().map(|output| output.value).sum::<u64>(),
        91_000
    );

    assert!(find_outputs_for_recipient(&transaction, "bc1qnobody").is_empty());
}

#[test]
fn bitcoin_value_trait_reports_sats_and_btc() {
    let payment = output(100_000_000, "bc1qreceiver");
    assert_eq!(payment.value(), 100_000_000);
    assert!((payment.value_in_btc() - 1.0).abs() < f64::EPSILON);

    // The same trait method works on either input variant.
    assert_eq!(regular_input(70_000).value(), 70_000);
    assert_eq!(coinbase_input(50_000).value(), 50_000);
}

#[test]
fn zero_value_output_is_allowed_for_op_return() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(48_000, "bc1qreceiver"));
    transaction.add_output(op_return(0));

    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn display_summarises_the_transaction() {
    let rendered = funded_transaction().to_string();

    assert!(rendered.contains("version 2"));
    assert!(rendered.contains("locktime 0"));
    assert!(rendered.contains("120000"));
    assert!(rendered.contains("118000"));
    assert!(rendered.contains("2000"));
}

#[test]
fn display_reports_an_invalid_fee_without_panicking() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));

    let rendered = transaction.to_string();
    assert!(rendered.contains("invalid"));
}

#[test]
fn outpoint_displays_as_txid_colon_vout() {
    let outpoint = OutPoint {
        txid: TXID.into(),
        vout: 3,
    };

    assert_eq!(outpoint.to_string(), format!("{TXID}:3"));
}

// --- Validation failures ---------------------------------------------------

#[test]
fn a_transaction_without_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(1_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn a_transaction_without_outputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
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
fn fee_does_not_underflow_when_outputs_exceed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));

    // Without checked subtraction this would wrap to a colossal u64.
    assert_eq!(
        transaction.fee(),
        Err(TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn a_zero_value_non_op_return_output_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn a_coinbase_input_cannot_be_mixed_with_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(312_500_000));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(100_000, "bc1qminer"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_are_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(312_500_000));
    transaction.add_input(coinbase_input(312_500_000));
    transaction.add_output(output(100_000, "bc1qminer"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn a_regular_input_with_an_empty_txid_is_rejected() {
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

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}

#[test]
fn every_error_has_a_useful_display_message() {
    let errors = [
        TransactionError::NoInputs,
        TransactionError::NoOutputs,
        TransactionError::ZeroValueOutput,
        TransactionError::OutputsExceedInputs {
            total_inputs: 1,
            total_outputs: 2,
        },
        TransactionError::CoinbaseMixedWithRegularInputs,
        TransactionError::MultipleCoinbaseInputs,
        TransactionError::InvalidTxid,
        TransactionError::InsufficientFunds {
            available: 1,
            required: 2,
        },
    ];

    for error in &errors {
        let message = error.to_string();
        assert!(!message.is_empty(), "{error:?} has no message");
        assert!(
            !message.contains("TODO"),
            "{error:?} still has a placeholder message"
        );
    }
}
