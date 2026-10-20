# Lab 04 — Inspect a UTXO and its outpoint

## Commands used

```bash
# 1. List unspent outputs for the miner wallet
bitcoin-cli -rpcwallet=miner listunspent

# 2. Inspect spendable UTXO details (txid, vout, amount, scriptPubKey, confirmations)
bitcoin-cli -rpcwallet=miner listunspent 1 9999999 '["bcrt1qmineraddress..."]'

# 3. Query total wallet balance
bitcoin-cli -rpcwallet=miner getbalance

# 4. Run Rust tests for Lab 04
cargo test --test lab_04
```

## Terminal output

```text
$ bitcoin-cli -rpcwallet=miner listunspent
[
  {
    "txid": "3e6220914e4112087c2167734b4562736aa720b5f7458fb886e6755834416922",
    "vout": 0,
    "address": "bcrt1qmineraddress",
    "label": "mining",
    "scriptPubKey": "0014aa27073f55bd7f026a4c494f68a8410342d17c4f",
    "amount": 50.00000000,
    "confirmations": 101,
    "spendable": true,
    "solvable": true,
    "safe": true
  }
]

$ cargo test --test lab_04
running 4 tests
test constructs_unique_outpoint ... ok
test decodes_listunspent_response ... ok
test selects_most_confirmed_spendable_utxo ... ok
test sums_only_spendable_outputs ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

![Polar UTXO Inspection Screenshot](evidence/lab01_05.png)

## Explanation

**UTXO Model vs Account Model Balance:**
- In Bitcoin, there is no centralized database column storing account balances. Instead, state is represented as a set of Unspent Transaction Outputs (UTXOs). Each UTXO is an immutable discrete coin locked by a `scriptPubKey`.
- An **OutPoint** (`txid:vout`) uniquely identifies a specific output in the UTXO set.
- A wallet balance is calculated dynamically by scanning the UTXO set, filtering for spendable outputs whose `scriptPubKey` matches keys or descriptors controlled by the wallet, and summing their individual `amount` values. The balance is not an account ledger entry; it is a calculated sum of discrete unspent outputs.
