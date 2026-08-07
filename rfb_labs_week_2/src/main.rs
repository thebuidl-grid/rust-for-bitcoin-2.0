//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // TODO(Part 8): add the two supplied UTXOs as inputs, then add the
    // payment and correctly calculated change outputs.
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "utxo1".to_string(),
            vout: 0,
        },
        value: 70000,
        sequence: 0xFFFFFFFF,
    });

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "utxo2".to_string(),
            vout: 1,
        },
        value: 50000,
        sequence: 0xFFFFFFFF,
    });

    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    });

    transaction.add_output(TxOutput {
        value: 28000,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2wpkh,
    });

    match transaction.fee() {
        Ok(fee) => println!("Calculated fee: {fee} sats"),
        Err(err) => println!("Fee Error: {err}"),
    }

    match transaction.validate() {
        Ok(()) => println!("Transaction is valid."),
        Err(err) => println!("Validation Error: {err}"),
    }

    println!("{transaction}");

    /*
    A test
    let mut tx = Transaction::new(2, 0);
       let input = InputKind::Coinbase { block_height: 1, reward: 5_000_000_000 };

       tx.add_input(input);
       println!("{}", input.value()); // triggers the error

     */
}
