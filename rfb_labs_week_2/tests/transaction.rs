use rfb_labs_week_2::{
    find_outputs_for_recipient, highest_value_output, InputKind, OutPoint, OutputType, Transaction,
    TxOutput,
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

fn output(value: u64, recipient: &str, output_type: OutputType) -> TxOutput {
    TxOutput {
        value,
        recipient: recipient.into(),
        output_type,
    }
}

#[test]
fn valid_regular_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver", OutputType::P2wpkh));
    transaction.add_output(output(28_000, "bc1qsender", OutputType::P2wpkh));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 120_000);
    assert_eq!(transaction.total_output_value(), 118_000);
    assert_eq!(transaction.fee(), Ok(2_000));
}

#[test]
fn outputs_cannot_exceed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver", OutputType::P2wpkh));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn empty_inputs_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qreceiver", OutputType::P2wpkh));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn empty_outputs_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn zero_value_output_without_opreturn_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_output(output(0, "bc1qreceiver", OutputType::P2wpkh));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::ZeroValueOutput)
    );
}

#[test]
fn opreturn_zero_value_output_is_valid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    transaction.add_output(output(0, "data", OutputType::OpReturn));
    transaction.add_output(output(9_000, "bc1qreceiver", OutputType::P2wpkh));
    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn mixed_coinbase_regular_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_input(coinbase_input(50_000));
    transaction.add_output(output(90_000, "bc1qreceiver", OutputType::P2wpkh));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn multiple_coinbase_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(coinbase_input(50_000));
    transaction.add_input(coinbase_input(50_000));
    transaction.add_output(output(90_000, "bc1qreceiver", OutputType::P2wpkh));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn empty_regular_txid_is_invalid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "".to_string(),
            vout: 0,
        },
        value: 10_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(9_000, "bc1qreceiver", OutputType::P2wpkh));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}

#[test]
fn highest_value_output_works() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(200_000));
    let o1 = output(50_000, "a", OutputType::P2wpkh);
    let o2 = output(100_000, "b", OutputType::P2wpkh);
    let o3 = output(30_000, "c", OutputType::P2wpkh);
    transaction.add_output(o1);
    transaction.add_output(o2);
    transaction.add_output(o3);

    let highest = highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 100_000);
    assert_eq!(highest.recipient, "b");
}

#[test]
fn find_outputs_for_recipient_works() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(200_000));
    transaction.add_output(output(50_000, "a", OutputType::P2wpkh));
    transaction.add_output(output(100_000, "b", OutputType::P2wpkh));
    transaction.add_output(output(30_000, "a", OutputType::P2wpkh));

    let matching = find_outputs_for_recipient(&transaction, "a");
    assert_eq!(matching.len(), 2);
    assert_eq!(matching[0].value, 50_000);
    assert_eq!(matching[1].value, 30_000);
}
