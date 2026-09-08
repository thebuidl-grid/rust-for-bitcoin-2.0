//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // TODO(Part 8): add the two supplied UTXOs as inputs, then add the
    // payment and correctly calculated change outputs.

    // Input 1: 70,000 sats UTXO
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });

    // Input 2: 50,000 sats UTXO
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            vout: 1,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    // Payment output: 90,000 sats to receiver
    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    });

    // Change output: 28,000 sats back to sender
    // fee = 70,000 + 50,000 - 90,000 - 28,000 = 2,000 sats
    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2wpkh,
    });

    println!("{transaction}");
}
