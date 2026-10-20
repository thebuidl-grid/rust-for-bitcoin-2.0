//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

const UTXO_ONE_VALUE: u64 = 70_000;
const UTXO_TWO_VALUE: u64 = 50_000;
const PAYMENT_VALUE: u64 = 90_000;
const TARGET_FEE: u64 = 2_000;
const CHANGE_VALUE: u64 = UTXO_ONE_VALUE + UTXO_TWO_VALUE - PAYMENT_VALUE - TARGET_FEE;

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // TODO(Part 8): add the two supplied UTXOs as inputs, then add the
    // payment and correctly calculated change outputs.

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value: UTXO_ONE_VALUE,
        sequence: u32::MAX,
    });

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            vout: 1,
        },
        value: UTXO_TWO_VALUE,
        sequence: u32::MAX,
    });

    transaction.add_output(TxOutput {
        value: PAYMENT_VALUE,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });

    transaction.add_output(TxOutput {
        value: CHANGE_VALUE,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    });

    match transaction.validate() {
        Ok(()) => println!("Transaction is valid!"),
        Err(error) => println!("Transaction is invalid: {error}"),
    }

    println!("{transaction}");
}
