//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // UTXO 1: 70,000 sats
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "1111111111111111111111111111111111111111111111111111111111111111".into(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });

    // UTXO 2: 50,000 sats
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "2222222222222222222222222222222222222222222222222222222222222222".into(),
            vout: 1,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    // Payment output: 90,000 sats to bc1qreceiver
    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });

    // Change output: 28,000 sats to bc1qsender (leaving 2,000 sats fee)
    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    });

    if let Err(e) = transaction.validate() {
        eprintln!("Transaction validation failed: {e}");
    }

    println!("{transaction}");
}
