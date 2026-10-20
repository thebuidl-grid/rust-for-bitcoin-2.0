//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    // Two UTXOs fund the payment: 70,000 + 50,000 = 120,000 sats.
    let first = InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    };
    let second = InputKind::Regular {
        previous_output: OutPoint {
            txid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            vout: 1,
        },
        value: 50_000,
        sequence: u32::MAX,
    };

    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(first);
    transaction.add_input(second);

    // Send 90,000 sats to the receiver and the remaining 28,000 sats
    // (120,000 - 90,000 - 2,000 fee) back as change to the sender.
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
