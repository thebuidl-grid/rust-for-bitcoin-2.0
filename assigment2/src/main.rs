use assigment2::{
    Address, CoinSelectionStrategy, OutPoint, Transaction, TransactionStatus, TxInput, TxOutput,
    UtxoSet, Validate, Wallet,
};

fn main() {
    // A coinbase transaction: the special input that mints a block reward
    // out of thin air rather than spending an existing UTXO.
    let mut coinbase = Transaction::new("coinbase-1", 0);
    coinbase.add_input(TxInput::Coinbase {
        block_height: 901_450,
    });
    coinbase.add_output(TxOutput {
        value: 50_000,
        address: Address::from("bc1q-alice"),
    });
    coinbase.validate().unwrap();
    println!("Mined:");
    print!("{coinbase}");

    // `confirmed_utxos` stands in for the blockchain's UTXO index: the source
    // of truth for "does this input actually point at spendable value?".
    let mut confirmed_utxos = UtxoSet::new();
    confirmed_utxos.insert(OutPoint::new("coinbase-1", 0), coinbase.outputs[0].clone());
    confirmed_utxos.insert(
        OutPoint::new("coinbase-2", 0),
        TxOutput {
            value: 12_000,
            address: Address::from("bc1q-alice"),
        },
    );
    confirmed_utxos.insert(
        OutPoint::new("coinbase-3", 0),
        TxOutput {
            value: 900,
            address: Address::from("bc1q-alice"),
        },
    );

    let mut wallet = Wallet::new(Address::from("bc1q-alice-change"));
    wallet.fund(OutPoint::new("coinbase-1", 0), coinbase.outputs[0].clone());
    wallet.fund(
        OutPoint::new("coinbase-2", 0),
        TxOutput {
            value: 12_000,
            address: Address::from("bc1q-alice"),
        },
    );
    wallet.fund(
        OutPoint::new("coinbase-3", 0),
        TxOutput {
            value: 900,
            address: Address::from("bc1q-alice"),
        },
    );

    println!("Starting balance: {} sats\n", wallet.balance());

    let payment = TxOutput {
        value: 20_000,
        address: Address::from("bc1q-bob"),
    };
    let fee = 250;

    match wallet.create_transaction(vec![payment], fee, CoinSelectionStrategy::LargestFirst) {
        Ok(mut tx) => {
            println!("Built payment to Bob:");
            print!("{tx}");
            println!(
                "Inputs: {} sats, fee: {} sats",
                tx.total_input_value(&confirmed_utxos).unwrap(),
                tx.fee(&confirmed_utxos).unwrap()
            );

            tx.advance_status(TransactionStatus::Signed).unwrap();
            tx.advance_status(TransactionStatus::Broadcast).unwrap();
            tx.advance_status(TransactionStatus::Confirmed { height: 901_452 })
                .unwrap();
            println!("Status: {}\n", tx.status());
        }
        Err(err) => println!("Failed to build payment: {err}\n"),
    }

    println!("Remaining balance: {} sats\n", wallet.balance());

    let oversized_payment = TxOutput {
        value: 1_000_000,
        address: Address::from("bc1q-carol"),
    };
    match wallet.create_transaction(
        vec![oversized_payment],
        fee,
        CoinSelectionStrategy::SmallestFirst,
    ) {
        Ok(tx) => print!("{tx}"),
        Err(err) => println!("Expected failure paying Carol: {err}"),
    }
}
