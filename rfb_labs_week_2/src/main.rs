//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "utxo-one-txid".into(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "utxo-two-txid".into(),
            vout: 1,
        },
        value: 50_000,
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

    println!("{transaction}");
}
