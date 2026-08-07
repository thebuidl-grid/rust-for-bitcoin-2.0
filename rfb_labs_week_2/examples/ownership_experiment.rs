//! Part 7 ownership experiment.
//!
//! Uncomment the line marked below and `cargo build --example
//! ownership_experiment` fails with E0382. The recorded error is in README.md.

use rfb_labs_week_2::{BitcoinValue, InputKind, OutPoint, Transaction};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    let input = InputKind::Regular {
        previous_output: OutPoint {
            txid: "9f2c4a1b3e5d70819f2c4a1b3e5d70819f2c4a1b3e5d70819f2c4a1b3e5d7081".to_string(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    };

    // `add_input` takes `InputKind` by value, so this moves `input` into the
    // transaction. `input` is dead from here on.
    transaction.add_input(input);

    // ✗ Uncommenting this is the experiment — E0382, borrow of moved value:
    // println!("{}", input.value());

    // ✓ The fix is to read the value back through a borrow of its new owner,
    // which is exactly what the Part 7 helpers do.
    if let Some(moved_input) = transaction.inputs.first() {
        println!(
            "the transaction now owns an input worth {}",
            moved_input.value()
        );
    }
}
