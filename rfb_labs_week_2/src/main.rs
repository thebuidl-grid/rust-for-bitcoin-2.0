//! Small executable for Part 8 of the assignment.
use rfb_labs_week_2::{
    transaction::Transaction,
    InputKind, OutPoint, OutputType, TxOutput,
};

fn main() {
    let mut tx = Transaction::new(2, 0);

    tx.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: 0xffffffff,
    });

    tx.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            vout: 1,
        },
        value: 50_000,
        sequence: 0xffffffff,
    });

    tx.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    });

    tx.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2wpkh,
    });

    match tx.validate() {
        Ok(()) => {
            println!("✓ Transaction is valid");
            println!("{}", tx);
        }
        Err(e) => {
            println!("✗ Transaction validation failed: {}", e);
        }
    }

    // let input_example = InputKind::Regular {
    //     previous_output: OutPoint {
    //         txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    //         vout: 0,
    //     },
    //     value: 70_000,
    //     sequence: 0xffffffff,
    // };
    // tx.add_input(input_example);

    // println!("{}", input_example);
}
