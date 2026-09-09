//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{OutPoint, OutputType, Transaction, TxOutput};
use rfb_labs_week_2::InputKind;

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

    let total_input = transaction.total_input_value();
    let payment = 90_000;
    let fee = 2_000;
    let change = total_input - payment - fee;

    transaction.add_output(TxOutput {
        value: payment,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });
    transaction.add_output(TxOutput {
        value: change,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    });

    match transaction.validate() {
        Ok(()) => println!("transaction is valid"),
        Err(error) => println!("transaction is invalid: {error}"),
    }

    println!("{transaction}");
}
