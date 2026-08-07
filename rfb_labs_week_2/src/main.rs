//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::{transaction::Transaction, InputKind, OutPoint, OutputType, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // Add two UTXOs as inputs: 70,000 and 50,000 sats
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "previous_tx_1".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: 0xffffffff,
    });

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "previous_tx_2".to_string(),
            vout: 1,
        },
        value: 50_000,
        sequence: 0xffffffff,
    });

    // Add payment output: 90,000 sats to receiver
    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    });

    // Calculate change: (70,000 + 50,000) - 90,000 - 2,000 fee = 28,000 sats
    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2wpkh,
    });

    println!("{transaction}");
}
