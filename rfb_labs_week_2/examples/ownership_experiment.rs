//! Part 7 ownership experiment.
//!
//! Uncomment the marked line to reproduce the borrow-after-move error. The exact
//! compiler output is recorded in `README.md` under "Written answers".
//!
//! Run with: `cargo run --example ownership_experiment`

use rfb_labs_week_2::{BitcoinValue, InputKind, OutPoint, Transaction};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    let input = InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaa".into(),
            vout: 0,
        },
        value: 120_000,
        sequence: u32::MAX,
    };

    // `input` is moved here. `InputKind` owns a `String` inside its `OutPoint`,
    // so it cannot be `Copy`, and the local variable is dead from this point on.
    transaction.add_input(input);

    // Uncommenting the next line fails to compile with E0382:
    // println!("{}", input.value());

    // Reading the same value through the transaction is fine, because the
    // transaction now owns it and we are only borrowing.
    if let Some(moved_input) = transaction.inputs.first() {
        println!(
            "value now owned by the transaction: {}",
            moved_input.value()
        );
    }
}
