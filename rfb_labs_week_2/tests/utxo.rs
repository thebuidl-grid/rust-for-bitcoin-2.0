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
fn selection_stops_as_soon_as_the_target_is_covered() {
    let available = vec![utxo("a", 70_000), utxo("b", 50_000), utxo("c", 10_000)];
    let selected = select_utxos(&available, 60_000).unwrap();

    // The first UTXO alone covers the target, so the others are never touched.
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value, 70_000);
}

#[test]
fn selection_returns_references_into_the_original_slice() {
    let available = vec![utxo("a", 70_000), utxo("b", 50_000)];
    let selected = select_utxos(&available, 100_000).unwrap();

    // Borrowed, not cloned: the addresses are the same.
    assert!(std::ptr::eq(selected[0], &available[0]));
    assert!(std::ptr::eq(selected[1], &available[1]));
}

#[test]
fn an_empty_wallet_cannot_fund_anything() {
    let available: Vec<Utxo> = Vec::new();

    assert_eq!(
        select_utxos(&available, 1),
        Err(TransactionError::InsufficientFunds {
            available: 0,
            required: 1,
        })
    );
}

#[test]
fn an_exact_match_selects_only_what_is_needed() {
    let available = vec![utxo("a", 90_000), utxo("b", 50_000)];
    let selected = select_utxos(&available, 90_000).unwrap();

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value, 90_000);
}

#[test]
fn a_zero_target_selects_nothing() {
    let available = vec![utxo("a", 70_000)];
    let selected = select_utxos(&available, 0).unwrap();

    assert!(selected.is_empty());
}
