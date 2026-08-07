//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // Add the two supplied UTXOs as inputs (70,000 and 50,000 sats)
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    // Total inputs: 120,000 sats
    // Payment: 90,000 sats to bc1qreceiver
    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    });

    // Change: 120,000 - 90,000 - 2,000 (fee) = 28,000 sats to bc1qsender
    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2wpkh,
    });

    println!("{transaction}");
}
