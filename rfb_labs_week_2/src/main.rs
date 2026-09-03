//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);
    // TODO(Part 8): add the two supplied UTXOs as inputs, then add the
    // payment and correctly calculated change outputs.
    let input_1 = InputKind::Regular {
        previous_output: OutPoint {
            txid: "txid_1".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: 0xffff_ffff,
    };

    let input_2 = InputKind::Regular {
        previous_output: OutPoint {
            txid: "txid_2".to_string(),
            vout: 1,
        },
        value: 50_000,
        sequence: 0xffff_ffff,
    };

    transaction.add_input(input_1);
    // println!("{input_1}");

    transaction.add_input(input_2);

    //Payment ouuput: 90 000 sats
    let payment_output = TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    };

    transaction.add_output(payment_output);

    let change_output = TxOutput {
    value: 28_000,
    recipient: "bc1qsender".to_string(),
    output_type: OutputType::P2wpkh,
};

transaction.add_output(change_output);

    match transaction.validate() {
        Ok(()) => {
            println!("{transaction}");
        }
        Err(error) => {
            eprintln!("Transaction validation failed: {error}");
        }
    }
}
