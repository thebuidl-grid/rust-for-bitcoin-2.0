use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    // Spend UTXOs of 70,000 and 50,000 sats
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            vout: 1,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    // Pay 90,000 sats to bc1qreceiver
    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });

    // Return change of 28,000 sats to bc1qsender (120,000 total - 90,000 payment - 2,000 fee)
    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".into(),
        output_type: OutputType::P2wpkh,
    });

    // Validate and print transaction summary
    if let Err(e) = transaction.validate() {
        println!("Validation Error: {}", e);
    } else {
        println!("{}", transaction);
    }
}
