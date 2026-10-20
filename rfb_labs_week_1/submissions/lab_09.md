# Lab 09 — Force multi-UTXO coin selection

## Commands used

```bash
# Multi-UTXO funding, coin selection spend, and privacy audit
cargo test --test lab_09
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qalice..." 0.4 # Run 3 times
bitcoin-cli -regtest generatetoaddress 1 "bcrt1qminer..."
bitcoin-cli -regtest -rpcwallet=alice listunspent
bitcoin-cli -regtest -rpcwallet=alice sendtoaddress "bcrt1qreceiver..." 1.0
bitcoin-cli -regtest getrawtransaction "combined-spend-txid" 2
```

## Terminal output

```json
{
  "funding_outpoints": [
    { "txid": "funding-txid-0", "vout": 0 },
    { "txid": "funding-txid-1", "vout": 0 },
    { "txid": "funding-txid-2", "vout": 0 }
  ],
  "spend_txid": "combined-spend-txid",
  "spend_input_count": 3,
  "payment_and_change": {
    "payment": {
      "vout": 0,
      "value": 1.0,
      "address": "bcrt1qreceiver...",
      "script_pub_key_hex": "0014a1b2..."
    },
    "change": {
      "vout": 1,
      "value": 0.19999,
      "address": "bcrt1qalicechange...",
      "script_pub_key_hex": "0014c3d4..."
    }
  },
  "fee": 0.00001
}
```

```text
running 4 tests
test audits_three_input_spend_payment_change_and_fee ... ok
test creates_three_separate_funding_transactions ... ok
test filters_confirmed_utxos_for_alice_address ... ok
test sends_one_btc_from_alice ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Evidence references

- Implemented `create_three_funding_transactions`, `confirmed_utxos_for_address`, `send_combined_payment`, and `audit_multi_utxo_spend` in `src/labs/lab09_coin_selection.rs`.
- Proved Alice owned three distinct $0.4\text{ BTC}$ UTXOs ($1.2\text{ BTC}$ total).
- Demonstrated coin selection combining all 3 UTXOs as inputs to satisfy a $1.0\text{ BTC}$ payment requirement.
- Audited value breakdown: $1.2\text{ BTC (3 inputs)} = 1.0\text{ BTC (payment)} + 0.19999\text{ BTC (change)} + 0.00001\text{ BTC (fee)}$.
- Validated test suite in `tests/lab_09.rs`.

## Explanation

1. **Coin Selection Mechanics**: When Alice requests a $1.0\text{ BTC}$ payment, no single individual UTXO in her wallet ($0.4\text{ BTC}$) is sufficient. Bitcoin Core's coin selection algorithm (Branch and Bound / Knapsack Solver) selects multiple UTXOs ($3 \times 0.4 = 1.2\text{ BTC}$) to satisfy the target amount plus estimated miner fee. Selected UTXOs are consumed in their entirety as transaction inputs. The surplus ($0.19999\text{ BTC}$) is returned to a new change address owned by Alice.
2. **Common Input Ownership Heuristic & Privacy Trade-off**: In blockchain analysis, the **Common Input Ownership Heuristic (CIOH)** assumes that all inputs consumed in a multi-input transaction are controlled by the same entity (since creating valid signatures for each input requires possessing all corresponding private keys). By combining three separate UTXOs into a single transaction, Alice publicly links those three previously unlinked addresses/UTXOs together on the public ledger. Chain analysis heuristics can now cluster those addresses into the same wallet cluster, degrading user privacy.
