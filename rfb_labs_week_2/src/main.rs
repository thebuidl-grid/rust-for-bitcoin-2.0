use rfb_labs_week_2::transaction::Transaction;

fn main() {
    let mut transaction = Transaction::new(2, 0);

    transaction.add_input(rfb_labs_week_2::InputKind::Regular {
        previous_output: rfb_labs_week_2::OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });

    transaction.add_input(rfb_labs_week_2::InputKind::Regular {
        previous_output: rfb_labs_week_2::OutPoint {
            txid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    transaction.add_output(rfb_labs_week_2::TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".into(),
        output_type: rfb_labs_week_2::OutputType::P2wpkh,
    });

    transaction.add_output(rfb_labs_week_2::TxOutput {
        value: 28_000,
        recipient: "bc1qsender".into(),
        output_type: rfb_labs_week_2::OutputType::P2wpkh,
    });

    println!("{transaction}");
}
