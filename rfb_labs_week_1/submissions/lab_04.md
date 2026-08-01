# Lab 04 — UTXOs and outpoints

## Commands used

```
cargo test --test lab_04
cargo fmt --check
BITCOIN_CLI=<bitcoin-cli wrapper> cargo run --example lab04_demo
```

Underlying RPCs (`src/labs/lab04_utxos.rs`):
```
listunspent            -rpcwallet=miner
getbalance              -rpcwallet=miner   # cross-check only, not part of the lab function
```

## Terminal output

`cargo test --test lab_04`:
```
running 4 tests
test constructs_unique_outpoint ... ok
test decodes_listunspent_response ... ok
test selects_most_confirmed_spendable_utxo ... ok
test sums_only_spendable_outputs ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab04_demo` against the live node (state carried over
from Lab 03: one matured 50 BTC coinbase, everything else still immature and
therefore absent from `listunspent`'s spendable set):
```
miner UTXOs (1 total):
  Utxo {
    txid: "8309b0a666fc79ec679cc77bc44d5ac3cda3962c27f991a7f35b4b8912f606bd",
    vout: 0,
    address: Some("bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv"),
    script_pub_key: "00149163a70141a18547fc27e4cdbea84f5acea6ac04",
    amount: 50.0,
    confirmations: 101,
    spendable: true
  }

selected spendable UTXO = (same as above)
its outpoint            = OutPoint { txid: "8309b0a6...606bd", vout: 0 }
sum of spendable UTXOs   = 50
```

Cross-check: `bitcoin-cli -rpcwallet=miner getbalance` → `50.00000000`.

## Evidence references

- Screenshot: `submissions/images/Screenshot from 2026-08-01 13-58-25.png` — IDE
  terminal running `cargo test --test lab_04`, all 4 tests passing.
- `txid` = `8309b0a666fc79ec679cc77bc44d5ac3cda3962c27f991a7f35b4b8912f606bd`,
  `vout` = `0`, `amount` = `50.0`, `confirmations` = `101`,
  `address` = `bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv`,
  `script_pub_key` = `00149163a70141a18547fc27e4cdbea84f5acea6ac04`,
  `spendable` = `true`.
- `outpoint()` combines the txid and vout into `OutPoint { txid, vout: 0 }` —
  the unique coordinate needed to reference this exact output as a future
  transaction input.
- `sum_spendable_utxos` = `50`, which matches `bitcoin-cli getbalance`'s
  `50.00000000` exactly, reconciling the independently-computed UTXO sum with
  Bitcoin Core's own reported wallet balance.

## Explanation

A wallet's "balance" is not a stored ledger entry the way a bank account
balance is — Bitcoin Core computes it on the fly each time by scanning every
UTXO (unspent transaction output) the wallet's keys can spend and summing
their amounts. In this run that sum is trivial (exactly one UTXO), but in
general a wallet balance is always a *derived* aggregate over a scattered set
of independent outputs, each with its own txid, vout, script, and
confirmation depth — never a single row that gets incremented or decremented.

The `OutPoint { txid, vout }` pair is what actually gives an individual UTXO
identity on the chain: a transaction can create many outputs, so `txid` alone
is ambiguous — `vout` disambiguates *which* output of that transaction is
being referenced. Every future input in the system spends by outpoint, not by
address or amount, which is why `outpoint()` exists as its own function
independent of the balance math.
