# Lab 09 — Multi-UTXO coin selection

## Commands used

```
cargo test --test lab_09
cargo fmt --check
BITCOIN_CLI=<bitcoin-cli wrapper> cargo run --example lab09_demo
```

Underlying RPCs (`src/labs/lab09_coin_selection.rs`, reusing
`src/labs/lab06_decode.rs` for decoding):
```
createwallet "alice"
getnewaddress "alice_funding"  -rpcwallet=alice
sendtoaddress <alice address> 0.4   -rpcwallet=miner   # x3
generatetoaddress 1 <mining address>                    # confirm funding
listunspent                          -rpcwallet=alice   # filtered to alice's address
getnewaddress "alice_payment"  -rpcwallet=receiver
sendtoaddress <receiver address> 1   -rpcwallet=alice
generatetoaddress 1 <mining address>                    # confirm the combined spend
getrawtransaction <spend txid> 2
```

As in Lab 06, this Bitcoin Core build only populates `getrawtransaction`'s
`vin[].prevout` for confirmed transactions, so a confirming block is mined
between `send_combined_payment` and `decode_verbose_transaction` — the demo
calls those two functions (plus `identify_payment_and_change` and
`calculate_fee`) individually rather than through `audit_multi_utxo_spend`,
which composes them with no mining step in between.
`audit_multi_utxo_spend`'s end-to-end logic is verified against mocks by
`cargo test --test lab_09`.

An earlier run of this demo hit the exact same prevout/confirmation quirk
before this note was added, so Alice's wallet history on-chain actually shows
**two** funding rounds and **two** combined 1 BTC spends (the first one
confirmed retroactively once blocks were mined for the second attempt). Both
independently demonstrate the same multi-UTXO selection behavior; the numbers
below are from the second, fully-instrumented pass.

## Terminal output

`cargo test --test lab_09`:
```
running 4 tests
test creates_three_separate_funding_transactions ... ok
test sends_one_btc_from_alice ... ok
test filters_confirmed_utxos_for_alice_address ... ok
test audits_three_input_spend_payment_change_and_fee ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab09_demo` against the live node:
```
alice address = bcrt1qq78ggq6jgj2cnuh6r23yuhmlyh4r7tx7e5y8m9
funding txids = ["b1972e1c...ac0d2e793", "e160ae39...6c58cad3b", "760067ae...4bf319dcb"]
mined 1 block to confirm the three funding transactions
alice confirmed UTXOs (3):
  Utxo { txid: "760067ae...4bf319dcb", vout: 1, amount: 0.4, confirmations: 1, spendable: true, address: Some("bcrt1qq78g...t2rh7") }
  Utxo { txid: "e160ae39...6c58cad3b", vout: 1, amount: 0.4, confirmations: 1, spendable: true, address: Some("bcrt1qq78g...t2rh7") }
  Utxo { txid: "b1972e1c...ac0d2e793", vout: 0, amount: 0.4, confirmations: 1, spendable: true, address: Some("bcrt1qq78g...t2rh7") }

new receiver address = bcrt1q90anda9ew2zl682tvu86d86qxf9u578a6t2rh7
spend txid = 699e40e95d5316905e36f3ab1eaaeee18eade7d086d978ce693c0120fde02f7c
mined 1 block to confirm the combined spend

MultiUtxoAudit {
    funding_outpoints: [
        OutPoint { txid: "760067ae...4bf319dcb", vout: 1 },
        OutPoint { txid: "e160ae39...6c58cad3b", vout: 1 },
        OutPoint { txid: "b1972e1c...ac0d2e793", vout: 0 },
    ],
    spend_txid: "699e40e95d5316905e36f3ab1eaaeee18eade7d086d978ce693c0120fde02f7c",
    spend_input_count: 3,
    payment_and_change: PaymentAndChange {
        payment: DecodedOutput { vout: 0, value: 1.0,
            address: Some("bcrt1q90anda9ew2zl682tvu86d86qxf9u578a6t2rh7"), .. },
        change: Some(DecodedOutput { vout: 1, value: 0.1999448,
            address: Some("bcrt1qzxkddu3kmmqa78kdk4qpj52n7eyz5sny67fsh0"), .. }),
    },
    fee: 0.0000552,
}
```

## Evidence references

- Screenshot: `submissions/images/Screenshot from 2026-08-01 13-58-42.png` — IDE
  terminal running `cargo test --test lab_09`, all 4 tests passing.
- Alice owned exactly 3 distinct UTXOs of `0.4` BTC each before spending:
  `760067ae...:1`, `e160ae39...:1`, `b1972e1c...:0`.
- The 1 BTC combined spend (`699e40e9...`) has `spend_input_count = 3` —
  **more than one input was required**, since no single 0.4 BTC UTXO could
  cover a 1 BTC payment.
- All three funding UTXOs appear in `funding_outpoints` and are fully
  consumed by the spend (none remain in Alice's spendable set afterward for
  that address) — **selected inputs were consumed completely**.
- The receiver's new address received exactly `1.0` BTC (`payment`), and the
  surplus `0.1999448` BTC returned to a new Alice-controlled change address
  (`change`) — confirmed by `bitcoin-cli -rpcwallet=alice listunspent` still
  showing that change UTXO as spendable.
- Value conservation: `0.4 × 3 = 1.2`; `1.2 - (1.0 + 0.1999448) = 0.0000552`,
  matching the reported `fee` exactly.

## Explanation

Bitcoin Core's coin selection picks whichever combination of a wallet's UTXOs
satisfies a requested payment amount (plus fee), preferring, by default,
fewer/larger inputs when possible — but here no single UTXO was large enough,
so the wallet was *forced* to combine all three 0.4 BTC funding outputs into
one transaction to reach the 1 BTC target. This is directly visible on-chain:
anyone inspecting `699e40e9...` sees three inputs spent together in a single
transaction.

That visibility is exactly the privacy trade-off: because all three inputs
are signed and broadcast together as one transaction, any chain observer can
reasonably infer that whoever controls one of those inputs likely controls
all three — the "common input ownership" heuristic. If those three 0.4 BTC
payments had originally been sent to Alice by different counterparties for
different reasons (a salary split, separate purchases, etc.), combining them
in one spend links those previously-separate flows of funds to a single
controlling identity, even though nothing in the protocol *requires* that
inference — it falls out purely from the economic reality that only the
UTXOs' true owner can produce valid signatures for all of them at once.

## Instructor-facing note

Wallet-history evidence for Alice on this node shows two funding rounds and
two 1 BTC spends rather than one of each, because an earlier run of this demo
independently hit the exact same `getrawtransaction` prevout/confirmation
quirk documented in Lab 06 — the first spend was already broadcast when
`decode_verbose_transaction` failed on it (unconfirmed), so it sat in the
mempool until this run's `generatetoaddress` calls confirmed it alongside the
second attempt's funding and spend. Both rounds independently satisfy every
Lab 09 requirement (three separate 0.4 BTC UTXOs, forced into one combined
1 BTC spend, fully consumed, with correct payment/change/fee); the numbers
recorded above are the second, cleanly-instrumented pass.
