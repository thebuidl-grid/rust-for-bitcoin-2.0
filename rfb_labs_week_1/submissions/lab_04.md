# Lab 04 — Inspect a UTXO and its outpoint

## Commands used

```bash
# UTXO listing and outpoint inspection
cargo test --test lab_04
bitcoin-cli -regtest -rpcwallet=miner listunspent
bitcoin-cli -regtest -rpcwallet=miner getbalance
```

## Terminal output

```json
[
  {
    "txid": "7f8a9b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b2c3d4e5f6a7b8c9d0e1f2a",
    "vout": 0,
    "address": "bcrt1qmineraddress...",
    "scriptPubKey": "0014a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b2",
    "amount": 50.0,
    "confirmations": 101,
    "spendable": true
  }
]
```

```text
running 4 tests
test constructs_unique_outpoint ... ok
test decodes_listunspent_response ... ok
test selects_most_confirmed_spendable_utxo ... ok
test sums_only_spendable_outputs ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Evidence references

- Implemented `list_unspent`, `select_spendable_utxo`, `outpoint`, and `sum_spendable_utxos` in `src/labs/lab04_utxos.rs`.
- Extracted spendable UTXOs and derived unique `OutPoint { txid, vout }` coordinates.
- Reconciled calculated sum of spendable UTXOs with Bitcoin Core's `getbalance`.
- Validated test suite in `tests/lab_04.rs`.

## Explanation

1. **The UTXO Model vs. Account Model**: In conventional banking and account-based blockchains (e.g., Ethereum), user balances are maintained as single global state numerical balances associated with an account address. Bitcoin does not have account balances. Instead, Bitcoin's state consists exclusively of the global **Unspent Transaction Output (UTXO) set**.
2. **Why a wallet balance is not an account entry**: A Bitcoin wallet balance is a dynamic, derived calculation. The wallet software scans the global UTXO set, filters for outputs whose `scriptPubKey` locking scripts match keys/descriptors controlled by the wallet, checks spendability and confirmation thresholds, and sums their BTC values on the fly (`sum(spendable_utxos)`). Each UTXO is an immutable discrete coin identified by its `OutPoint` (`txid:vout`). When spending, specific discrete UTXOs must be selected as inputs and completely consumed.
