//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::{
    highest_value_output, select_utxos, InputKind, OutPoint, OutputType, Transaction, TxOutput,
    Utxo,
};

const PAYMENT: u64 = 90_000;
const TARGET_FEE: u64 = 2_000;
const RECIPIENT: &str = "bc1qreceiver";
const CHANGE_ADDRESS: &str = "bc1qsender";

fn main() {
    let wallet = vec![
        Utxo {
            outpoint: OutPoint {
                txid: "9f2c4a1b3e5d70819f2c4a1b3e5d70819f2c4a1b3e5d70819f2c4a1b3e5d7081"
                    .to_string(),
                vout: 0,
            },
            value: 70_000,
        },
        Utxo {
            outpoint: OutPoint {
                txid: "6d4e2f8a0c1b39576d4e2f8a0c1b39576d4e2f8a0c1b39576d4e2f8a0c1b3957"
                    .to_string(),
                vout: 1,
            },
            value: 50_000,
        },
    ];

    // The wallet must cover the payment *and* the fee it intends to leave.
    let selected = match select_utxos(&wallet, PAYMENT + TARGET_FEE) {
        Ok(selected) => selected,
        Err(error) => {
            eprintln!("cannot fund the payment: {error}");
            return;
        }
    };

    println!("Selected {} UTXO(s):", selected.len());
    for utxo in &selected {
        println!("  - {} worth {} sats", utxo.outpoint, utxo.value);
    }
    println!();

    let mut transaction = Transaction::new(2, 0);

    // Each selected UTXO becomes a regular input referencing the output it spends.
    for utxo in &selected {
        transaction.add_input(InputKind::Regular {
            previous_output: OutPoint {
                txid: utxo.outpoint.txid.clone(),
                vout: utxo.outpoint.vout,
            },
            value: utxo.value,
            sequence: u32::MAX,
        });
    }

    transaction.add_output(TxOutput {
        value: PAYMENT,
        recipient: RECIPIENT.to_string(),
        output_type: OutputType::P2wpkh,
    });

    // Change is derived rather than hardcoded: everything the inputs provide
    // that the payment and the intended fee do not claim comes back to us.
    // Anything left unclaimed by an output is what the miner collects, so the
    // fee is never an explicit field.
    let change = transaction
        .total_input_value()
        .saturating_sub(PAYMENT)
        .saturating_sub(TARGET_FEE);

    if change > 0 {
        transaction.add_output(TxOutput {
            value: change,
            recipient: CHANGE_ADDRESS.to_string(),
            output_type: OutputType::P2wpkh,
        });
    }

    if let Err(error) = transaction.validate() {
        eprintln!("refusing to broadcast an invalid transaction: {error}");
        return;
    }

    println!("{transaction}");

    if let Some(output) = highest_value_output(&transaction) {
        println!("\nLargest output: {output}");
    }
}
