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

fn coinbase_input(block_height: u32, reward: u64) -> InputKind {
    InputKind::Coinbase {
        block_height,
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

fn op_return_output(recipient: &str) -> TxOutput {
    TxOutput {
        value: 0,
        recipient: recipient.into(),
        output_type: OutputType::OpReturn,
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
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(840_000, 3_125_000_000));
    transaction.add_output(output(3_125_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 3_125_000_000);
    assert_eq!(transaction.total_output_value(), 3_125_000_000);
    assert_eq!(transaction.fee(), Ok(0));
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
fn error_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(50_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::NoInputs));
}

#[test]
fn error_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));

    assert_eq!(transaction.validate(), Err(TransactionError::NoOutputs));
}

#[test]
fn error_zero_value_non_op_return_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(0, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::ZeroValueOutput)
    );
}

#[test]
fn valid_zero_value_op_return_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(48_000, "bc1qreceiver"));
    transaction.add_output(op_return_output("OP_RETURN data"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.fee(), Ok(2_000));
}

#[test]
fn error_mixed_coinbase_and_regular_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(840_000, 50_000));
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(80_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn error_multiple_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(840_000, 50_000));
    transaction.add_input(coinbase_input(840_001, 50_000));
    transaction.add_output(output(80_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn error_empty_regular_input_txid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(48_000, "bc1qreceiver"));

    assert_eq!(transaction.validate(), Err(TransactionError::InvalidTxid));
}

#[test]
fn highest_value_output_and_recipient_queries() {
    let mut transaction = Transaction::new(2, 0);
    let out1 = output(30_000, "alice");
    let out2 = output(70_000, "bob");
    let out3 = output(20_000, "alice");

    transaction.add_output(out1);
    transaction.add_output(out2);
    transaction.add_output(out3);

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 70_000);
    assert_eq!(highest.recipient, "bob");

    let alice_outputs = find_outputs_for_recipient(&transaction, "alice");
    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs[0].value, 30_000);
    assert_eq!(alice_outputs[1].value, 20_000);
}

#[test]
fn bitcoin_value_trait_works() {
    let out = output(100_000_000, "alice");
    assert_eq!(out.value(), 100_000_000);
    assert_eq!(out.value_in_btc(), 1.0);

    let reg_in = regular_input(50_000_000);
    assert_eq!(reg_in.value(), 50_000_000);
    assert_eq!(reg_in.value_in_btc(), 0.5);

    let cb_in = coinbase_input(100, 250_000_000);
    assert_eq!(cb_in.value(), 250_000_000);
    assert_eq!(cb_in.value_in_btc(), 2.5);
}
