//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // Spend a 70,000-sat UTXO.
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "a1b2c3d4e5f60000000000000000000000000000000000000000000000aaaa".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: 0xffffffff,
    });

    // Spend a 50,000-sat UTXO.
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "b1c2d3e4f5060000000000000000000000000000000000000000000000bbbb".to_string(),
            vout: 1,
        },
        value: 50_000,
        sequence: 0xffffffff,
    });

    // Pay the receiver 90,000 sats.
    let payment = 90_000;
    transaction.add_output(TxOutput {
        value: payment,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    });

    // Return change to the sender, leaving exactly a 2,000-sat fee.
    let target_fee = 2_000;
    let change = transaction.total_input_value() - payment - target_fee;
    transaction.add_output(TxOutput {
        value: change,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2wpkh,
    });

    match transaction.validate() {
        Ok(()) => println!("Transaction is valid."),
        Err(err) => println!("Transaction is INVALID: {err}"),
    }

    match transaction.fee() {
        Ok(fee) => println!("Calculated fee: {fee} sats"),
        Err(err) => println!("Fee calculation failed: {err}"),
    }

    println!("{transaction}");
}
