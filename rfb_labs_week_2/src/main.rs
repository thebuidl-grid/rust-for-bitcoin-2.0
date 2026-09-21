//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::transaction::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // TODO(Part 8): add the two supplied UTXOs as inputs, then add the
    // payment and correctly calculated change outputs.

    // First Input of 70,000 sats
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "txid_of_the_first".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });

    // Second Input of 50,000 sats
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "txid_of_the_second".to_string(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    // Output of 90,000 sats
    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2pkh,
    });

    // Change  of 28,000 sats
    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2pkh,
    });

    // fee = Inpus - Outputs => ( (70,000 + 50,000 ) - (90,000 + 28,000) ) => 2000
    let fee = transaction.fee();

    assert_eq!(2000_u64, fee.expect("Invalid fee"));

    println!("{transaction}");
}
