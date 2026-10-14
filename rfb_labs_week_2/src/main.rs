//! Small executable for Part 8 and Part 10 of the assignment.

use rfb_labs_week_2::transaction::{
    InputKind, OutPoint, OutputType, Transaction, TxOutput, TxState,
};

fn main() {
    // 1. Build the Part 8 Transaction
    let mut transaction = Transaction::new(2, 0);

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: 0xFFFFFFFF,
    });

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            vout: 1,
        },
        value: 50_000,
        sequence: 0xFFFFFFFF,
    });

    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_string(),
        output_type: OutputType::P2wpkh,
    });

    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".to_string(),
        output_type: OutputType::P2wpkh,
    });

    // Part 8 Display Output
    println!("--- Part 8 Transaction ---");
    println!("{transaction}\n");

    // Part 10 Demonstration
    println!("--- Part 10 State Machine ---");
    let state_tx = TxState::new(transaction);

    match state_tx.validate() {
        Ok(validated_tx) => {
            println!("State: Validated");
            let signed_tx = validated_tx.sign("30440220...sig");
            println!("State: Signed (Sig: {})", signed_tx.state_data.signature);
            let broadcast_tx = signed_tx.broadcast("txid_abc123");
            println!("State: Broadcast (TXID: {})", broadcast_tx.state_data.txid);
            let confirmed_tx = broadcast_tx.confirm(840_000);
            println!(
                "State: Confirmed at block {}",
                confirmed_tx.state_data.block_height
            );
        }
        Err(rejected_tx) => {
            println!("Transaction Rejected: {}", rejected_tx.state_data.reason);
        }
    }
}
