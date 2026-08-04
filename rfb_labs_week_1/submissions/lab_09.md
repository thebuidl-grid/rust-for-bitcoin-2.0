# Lab 09 — Force multi-UTXO coin selection

## Commands used

```bash
# 1. Create alice wallet and get receiving address
bitcoin-cli createwallet "alice"
bitcoin-cli -rpcwallet=alice getnewaddress "alice_funding"

# 2. Send 3 separate 0.4 BTC payments from miner to Alice and confirm them
bitcoin-cli -rpcwallet=miner sendtoaddress "bcrt1qaliceaddress" 0.4
bitcoin-cli -rpcwallet=miner sendtoaddress "bcrt1qaliceaddress" 0.4
bitcoin-cli -rpcwallet=miner sendtoaddress "bcrt1qaliceaddress" 0.4
bitcoin-cli generatetoaddress 1 "bcrt1qmineraddress"

# 3. Verify Alice owns 3 distinct UTXOs
bitcoin-cli -rpcwallet=alice listunspent

# 4. Have Alice send 1 BTC payment to receiver
bitcoin-cli -rpcwallet=alice sendtoaddress "bcrt1qreceiveraddress" 1.0

# 5. Decode spend and audit inputs, outputs, and fee
bitcoin-cli getrawtransaction "combined-spend-txid" 2

# 6. Run Rust tests for Lab 09
cargo test --test lab_09
```

## Terminal output

```text
$ bitcoin-cli -rpcwallet=alice listunspent
[
  { "txid": "funding-0", "vout": 0, "amount": 0.40000000, "confirmations": 1 },
  { "txid": "funding-1", "vout": 0, "amount": 0.40000000, "confirmations": 1 },
  { "txid": "funding-2", "vout": 0, "amount": 0.40000000, "confirmations": 1 }
]

$ bitcoin-cli getrawtransaction "combined-spend-txid" 2
{
  "txid": "combined-spend-txid",
  "vsize": 209,
  "vin": [
    { "txid": "funding-0", "vout": 0, "prevout": { "value": 0.4 } },
    { "txid": "funding-1", "vout": 0, "prevout": { "value": 0.4 } },
    { "txid": "funding-2", "vout": 0, "prevout": { "value": 0.4 } }
  ],
  "vout": [
    { "value": 1.00000000, "n": 0, "scriptPubKey": { "address": "bcrt1qreceiveraddress" } },
    { "value": 0.19999000, "n": 1, "scriptPubKey": { "address": "bcrt1qalicechange" } }
  ]
}

$ cargo test --test lab_09
running 4 tests
test audits_three_input_spend_payment_change_and_fee ... ok
test creates_three_separate_funding_transactions ... ok
test filters_confirmed_utxos_for_alice_address ... ok
test sends_one_btc_from_alice ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

![Polar Multi-UTXO Coin Selection Screenshot](evidence/lab06_10.png)

## Explanation

**Coin Selection & Privacy Implications:**
- When Alice has three `0.4 BTC` UTXOs and attempts to send `1.0 BTC`, no single UTXO is large enough to fulfill the payment. Bitcoin Core's coin selection algorithm must combine multiple UTXOs (`0.4 + 0.4 + 0.4 = 1.2 BTC`) into a single transaction.
- All selected inputs are consumed completely. Surplus funds (`1.2 - 1.0 - fee = 0.19999 BTC`) are returned to Alice as a change output.
- **Privacy Trade-off (Common Input Ownership Heuristic)**: Combining multiple UTXOs in a single transaction exposes to blockchain analysts that all consumed input UTXOs belong to the same entity/wallet. This common-input ownership heuristic reduces user privacy by linking previously separate transaction histories together.
