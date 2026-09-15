//! Small executable for Part 8 of the assignment.
//!
//! Spends two UTXOs worth 70,000 and 50,000 sats, pays 90,000 sats to
//! `bc1qreceiver`, and returns the remainder to `bc1qsender` so that exactly
//! 2,000 sats are left over as the fee.

use rfb_labs_week_2::{
    highest_value_output, select_utxos, BitcoinValue, InputKind, OutPoint, OutputType, Transaction,
    TxOutput, Utxo,
};

const PAYMENT_SATS: u64 = 90_000;
const FEE_SATS: u64 = 2_000;
const RECEIVER: &str = "bc1qreceiver";
const SENDER: &str = "bc1qsender";

fn main() {
    // The wallet's spendable coins. Owned here, borrowed by the selector.
    let wallet = vec![
        Utxo {
            outpoint: OutPoint {
                txid: "1111111111111111111111111111111111111111111111111111111111111111".into(),
                vout: 0,
            },
            value: 70_000,
        },
        Utxo {
            outpoint: OutPoint {
                txid: "2222222222222222222222222222222222222222222222222222222222222222".into(),
                vout: 1,
            },
            value: 50_000,
        },
    ];

    let required = PAYMENT_SATS + FEE_SATS;
    let selected = match select_utxos(&wallet, required) {
        Ok(selected) => selected,
        Err(error) => {
            eprintln!("could not fund the payment: {error}");
            return;
        }
    };

    println!(
        "Selected {} UTXO(s) to cover {required} sats:",
        selected.len()
    );
    for utxo in &selected {
        println!("  {} worth {} sats", utxo.outpoint, utxo.value);
    }
    println!();

    let mut transaction = Transaction::new(2, 0);

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
        value: PAYMENT_SATS,
        recipient: RECEIVER.into(),
        output_type: OutputType::P2wpkh,
    });

    // Change is derived, never hardcoded: whatever is left after the payment and
    // the intended fee goes back to the sender.
    let change = transaction.total_input_value() - PAYMENT_SATS - FEE_SATS;
    transaction.add_output(TxOutput {
        value: change,
        recipient: SENDER.into(),
        output_type: OutputType::P2wpkh,
    });

    match transaction.validate() {
        Ok(()) => println!("Validation: passed"),
        Err(error) => println!("Validation: failed ({error})"),
    }

    println!("{transaction}");
    println!();

    for input in &transaction.inputs {
        println!("  input:  {input}");
    }
    for output in &transaction.outputs {
        println!("  output: {output}");
    }

    if let Some(largest) = highest_value_output(&transaction) {
        println!();
        println!(
            "Largest output: {largest} ({:.8} BTC)",
            largest.value_in_btc()
        );
    }
}
