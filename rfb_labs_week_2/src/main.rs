//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // Spend the first UTXO (70,000 sats)
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "utxo-1".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });

    // Spend the second UTXO (50,000 sats)
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "utxo-2".to_string(),
            vout: 1,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    // Payment output
    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    });

    // Change output
    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2wpkh,
    });

    println!("{transaction}");
}
