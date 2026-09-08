//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    let regular_input = InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    };

    let second_input = InputKind::Regular {
        previous_output: OutPoint {
            txid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    };

    let receiver_output = TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    };

    let sender_output = TxOutput {
        value: 28_000,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    };

    transaction.add_input(regular_input);
    transaction.add_input(second_input);
    transaction.add_output(receiver_output);
    transaction.add_output(sender_output);

    match transaction.validate() {
        Ok(()) => println!("{transaction}"),
        Err(error) => eprintln!("transaction validation failed: {error}"),
    }
}
