# Lab 04 — UTXOs and outpoints

## Commands used

Rust:

```
cargo test --test lab_04
cargo fmt --check
cargo run --example lab04
```

`examples/lab04.rs` calls the completed `list_unspent`, `select_spendable_utxo`, `outpoint`, and
`sum_spendable_utxos` functions against the `miner` wallet on the real node.

Bitcoin Core RPCs (run directly in Polar's node terminal):

```
bitcoin-cli -rpcwallet=miner listunspent
bitcoin-cli -rpcwallet=miner getbalance
```

## Terminal output

`cargo test --test lab_04`:

```
running 4 tests
test constructs_unique_outpoint ... ok
test sums_only_spendable_outputs ... ok
test selects_most_confirmed_spendable_utxo ... ok
test decodes_listunspent_response ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`cargo run --example lab04` (real node, via the completed Rust implementation):

```
miner wallet UTXO count: 101
selected UTXO: Utxo {
    txid: "1a89345e8580f9143a8a16df4224d50a60a7a9076ee24246d3b4273b318ffc41",
    vout: 0,
    address: Some(
        "bcrt1q0tvlxqh4vkfzwuu9qun9d4txwrf76uj7syyvhy",
    ),
    script_pub_key: "00147ad9f302f56592277385072656d56670d3ed725e",
    amount: 50.0,
    confirmations: 202,
    spendable: true,
}
its outpoint: OutPoint {
    txid: "1a89345e8580f9143a8a16df4224d50a60a7a9076ee24246d3b4273b318ffc41",
    vout: 0,
}
independently summed spendable balance: 5050
```

Raw `bitcoin-cli` output (cross-checking the same wallet):

```
$ bitcoin-cli -rpcwallet=miner listunspent
[
  {
    "txid": "1a89345e8580f9143a8a16df4224d50a60a7a9076ee24246d3b4273b318ffc41",
    "vout": 0,
    "address": "bcrt1q0tvlxqh4vkfzwuu9qun9d4txwrf76uj7syyvhy",
    "scriptPubKey": "00147ad9f302f56592277385072656d56670d3ed725e",
    "amount": 50.00000000,
    "confirmations": 202,
    "spendable": true,
    "solvable": true,
    ...
  },
  ... 100 more entries, all 50 BTC, all spendable ...
]

$ bitcoin-cli -rpcwallet=miner getbalance
5050.00000000
```

The `miner` wallet accumulated 101 separate coinbase UTXOs (1 from Lab 01/02 mining, then 100 more
from Lab 03's maturity demo, all mined to the same reused address), each worth exactly 50 BTC. The
Rust implementation's independently-computed sum of spendable UTXOs (`5050`) matches Bitcoin Core's
own `getbalance` report (`5050.00000000`) exactly.

## Evidence references

Terminal output above was captured directly from Polar's node terminal and from
`cargo run --example lab04`; no separate screenshots were taken for this lab.

## Explanation

A UTXO ("unspent transaction output") is a discrete, indivisible chunk of bitcoin sitting at a
specific output of a specific past transaction — not a row in some running ledger. Its unique
coordinate, an **outpoint**, is just `txid:vout`: the transaction that created it, plus which
output index within that transaction (a single transaction can create several outputs/UTXOs at
once, hence the index). Spending a UTXO means consuming that *entire* output as a transaction
input — there's no such thing as spending "part" of one; if you only need part of its value, the
rest comes back to you as a new change UTXO in the same spending transaction.

A wallet's balance is not a stored number anywhere — it's simply the **sum of every UTXO the wallet
currently controls the private key for**, filtered to whichever ones are actually spendable right
now. That's why `getbalance` and independently summing `listunspent`'s spendable entries land on
exactly the same figure here: there is no separate "account balance" ledger entry to reconcile
against, Bitcoin Core computes both the same way, from the same underlying UTXO set. This is a
fundamentally different accounting model from a bank account (a single mutable balance field) —
Bitcoin has no balances at all, only a scattered set of unspent outputs that a wallet happens to
recognize as its own and adds up on demand.
