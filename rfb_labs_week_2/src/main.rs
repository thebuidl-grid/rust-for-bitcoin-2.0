//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "1".repeat(64),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "2".repeat(64),
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

    let change = transaction.total_input_value() - 90_000 - 2_000;
    transaction.add_output(TxOutput {
        value: change,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    });

    match transaction.validate() {
        Ok(()) => println!("transaction is valid\n"),
        Err(err) => println!("transaction is invalid: {err}\n"),
    }

    println!("{transaction}");
}
