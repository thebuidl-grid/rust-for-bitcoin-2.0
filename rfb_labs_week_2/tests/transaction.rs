use rfb_labs_week_2::{
    find_outputs_for_recipient, highest_value_output, BitcoinValue, InputKind, OutPoint,
    OutputType, Transaction, TransactionError, TxOutput,
};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn regular_input(value: u64) -> InputKind {
    regular_input_with_txid(value, TXID)
}

fn regular_input_with_txid(value: u64, txid: &str) -> InputKind {
    InputKind::Regular {
        previous_output: OutPoint {
            txid: txid.into(),
            vout: 0,
        },
        value,
        sequence: u32::MAX,
    }
}

fn coinbase_input(reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height: 840_000,
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

fn op_return_output() -> TxOutput {
    TxOutput {
        value: 0,
        recipient: "OP_RETURN 6a0b68656c6c6f".into(),
        output_type: OutputType::OpReturn,
    }
}

// -- Parts 3 and 5: a well-formed transaction ------------------------------

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
fn totals_sum_every_input_and_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(70_000));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));
    transaction.add_output(output(30_000, "bc1qother"));
    transaction.add_output(output(28_000, "bc1qsender"));

    assert_eq!(transaction.inputs.len(), 2);
    assert_eq!(transaction.outputs.len(), 3);
    assert_eq!(transaction.total_input_value(), 120_000);
    assert_eq!(transaction.total_output_value(), 118_000);
    assert_eq!(transaction.fee(), Ok(2_000));
}

#[test]
fn a_transaction_that_spends_everything_has_a_zero_fee() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(transaction.fee(), Ok(0));
    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(312_500_000));
    transaction.add_output(output(312_500_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 312_500_000);
    assert_eq!(transaction.fee(), Ok(0));
}

// -- Part 5: every validation rule -----------------------------------------

#[test]
fn a_transaction_without_inputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn a_transaction_without_outputs_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn a_zero_value_output_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn a_zero_value_op_return_output_is_allowed() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(48_000, "bc1qreceiver"));
    transaction.add_output(op_return_output());

    assert_eq!(transaction.validate(), Ok(()));
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
fn a_coinbase_input_cannot_be_mixed_with_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(90_000, "bc1qminer"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_inputs_are_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(25_000));
    transaction.add_input(coinbase_input(25_000));
    transaction.add_output(output(40_000, "bc1qminer"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn a_regular_input_with_an_empty_txid_is_rejected() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input_with_txid(50_000, ""));
    transaction.add_output(output(40_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}

#[test]
fn fee_reports_an_error_instead_of_underflowing() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(1_000));
    transaction.add_output(output(5_000, "bc1qreceiver"));

    assert_eq!(
        transaction.fee(),
        Err(TransactionError::OutputsExceedInputs {
            total_inputs: 1_000,
            total_outputs: 5_000,
        })
    );
}

// -- Part 6: traits ---------------------------------------------------------

#[test]
fn bitcoin_value_reads_both_input_variants_and_outputs() {
    assert_eq!(regular_input(70_000).value(), 70_000);
    assert_eq!(coinbase_input(312_500_000).value(), 312_500_000);
    assert_eq!(output(90_000, "bc1qreceiver").value(), 90_000);
}

#[test]
fn bitcoin_value_converts_satoshis_to_btc() {
    assert_eq!(output(100_000_000, "bc1qreceiver").value_in_btc(), 1.0);
    assert_eq!(coinbase_input(312_500_000).value_in_btc(), 3.125);
}

#[test]
fn an_outpoint_displays_as_txid_colon_vout() {
    let outpoint = OutPoint {
        txid: TXID.into(),
        vout: 3,
    };

    assert_eq!(outpoint.to_string(), format!("{TXID}:3"));
}

#[test]
fn the_transaction_summary_reports_version_counts_and_fee() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    let summary = transaction.to_string();

    assert!(summary.contains("v2"));
    assert!(summary.contains("locktime 0"));
    assert!(summary.contains("1 input(s) totalling 120000 sats"));
    assert!(summary.contains("2 output(s) totalling 118000 sats"));
    assert!(summary.contains("fee: 2000 sats"));
}

#[test]
fn the_transaction_summary_reports_an_invalid_fee_without_panicking() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));

    let summary = transaction.to_string();

    assert!(summary.contains("fee: unavailable"));
    assert!(summary.contains("60000"));
}

// -- Part 7: borrowing ------------------------------------------------------

#[test]
fn highest_value_output_borrows_the_largest_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(28_000, "bc1qsender"));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(1_000, "bc1qdust"));

    let largest = highest_value_output(&transaction).expect("three outputs were added");

    assert_eq!(largest.value, 90_000);
    assert_eq!(largest.recipient, "bc1qreceiver");
    // The transaction still owns its outputs.
    assert_eq!(transaction.outputs.len(), 3);
}

#[test]
fn highest_value_output_is_none_without_outputs() {
    let transaction = Transaction::new(2, 0);

    assert!(highest_value_output(&transaction).is_none());
}

#[test]
fn find_outputs_for_recipient_returns_every_match() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));
    transaction.add_output(output(5_000, "bc1qreceiver"));

    let matches = find_outputs_for_recipient(&transaction, "bc1qreceiver");

    assert_eq!(matches.len(), 2);
    assert_eq!(
        matches.iter().map(|output| output.value).sum::<u64>(),
        95_000
    );
}

#[test]
fn find_outputs_for_recipient_is_empty_for_an_unknown_address() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(90_000, "bc1qreceiver"));

    assert!(find_outputs_for_recipient(&transaction, "bc1qstranger").is_empty());
}
