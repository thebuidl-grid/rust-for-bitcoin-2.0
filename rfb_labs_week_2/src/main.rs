//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::Transaction;
use rfb_labs_week_2::{InputKind, OutPoint, OutputType, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // TODO(Part 8): add the two supplied UTXOs as inputs, then add the
    // payment and correctly calculated change outputs.
    transaction.add_input(regular_input(70_000));
    transaction.add_input(regular_input(50_000));

    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });

    let change = transaction.total_input_value() - 90_000 - 2000;

    transaction.add_output(TxOutput {
        value: change,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    });
    let _ = transaction.validate();
    println!("{transaction}");
}

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
