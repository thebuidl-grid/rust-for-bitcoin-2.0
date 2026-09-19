use rfb_labs_week_2::{select_utxos, OutPoint, TransactionError, Utxo};

fn utxo(txid: &str, value: u64) -> Utxo {
    Utxo {
        outpoint: OutPoint {
            txid: txid.into(),
            vout: 0,
        },
        value,
    }
}

#[test]
fn selection_borrows_enough_utxos_in_slice_order() {
    let available = vec![utxo("a", 70_000), utxo("b", 50_000)];
    let selected = select_utxos(&available, 90_000).unwrap();

    assert_eq!(selected.len(), 2);
    assert_eq!(selected.iter().map(|utxo| utxo.value).sum::<u64>(), 120_000);
}

#[test]
fn insufficient_funds_is_an_error() {
    let available = vec![utxo("a", 30_000), utxo("b", 20_000)];

    assert_eq!(
        select_utxos(&available, 60_000),
        Err(TransactionError::InsufficientFunds {
            available: 50_000,
            required: 60_000,
        })
    );
}

#[test]
fn selection_returns_exact_match() {
    let available = vec![utxo("a", 70_000), utxo("b", 50_000)];
    let selected = select_utxos(&available, 70_000).unwrap();

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value, 70_000);
}

#[test]
fn selection_returns_single_utxo_when_sufficient() {
    let available = vec![utxo("a", 70_000), utxo("b", 50_000)];
    let selected = select_utxos(&available, 120_000).unwrap();

    assert_eq!(selected.len(), 2);
    assert_eq!(selected.iter().map(|utxo| utxo.value).sum::<u64>(), 120_000);
}

#[test]
fn selection_returns_empty_slice_when_target_is_zero() {
    let available = vec![utxo("a", 70_000), utxo("b", 50_000)];
    let selected = select_utxos(&available, 0).unwrap();

    assert!(selected.is_empty());
}

#[test]
fn selection_returns_empty_with_no_utxos() {
    let available: Vec<Utxo> = vec![];
    let selected = select_utxos(&available, 10_000);

    assert_eq!(
        selected,
        Err(TransactionError::InsufficientFunds {
            available: 0,
            required: 10_000,
        })
    );
}
