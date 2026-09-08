//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "1111111111111111111111111111111111111111111111111111111111111111".into(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "2222222222222222222222222222222222222222222222222222222222222222".into(),
            vout: 1,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    const PAYMENT: u64 = 90_000;
    const FEE: u64 = 2_000;
    let change = transaction.total_input_value() - PAYMENT - FEE;

    transaction.add_output(TxOutput {
        value: PAYMENT,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });
    transaction.add_output(TxOutput {
        value: change,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    });

    match transaction.validate() {
        Ok(()) => println!("Transaction is valid."),
        Err(err) => println!("Transaction is invalid: {err}"),
    }

    println!("{transaction}");
}
