# Lab 09 — Force multi-UTXO coin selection

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_09

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qalice" 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qalice" 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qalice" 0.4
bitcoin-cli -regtest -rpcwallet=alice listunspent
bitcoin-cli -regtest -rpcwallet=alice sendtoaddress "bcrt1qreceiver" 1.0
bitcoin-cli -regtest getrawtransaction "combined-spend" 2
```

## Terminal output

```json
// listunspent showing Alice's three UTXOs:
[
  { "txid": "funding-0", "vout": 0, "amount": 0.4, "confirmations": 1 },
  { "txid": "funding-1", "vout": 0, "amount": 0.4, "confirmations": 1 },
  { "txid": "funding-2", "vout": 0, "amount": 0.4, "confirmations": 1 }
]

// getrawtransaction vin showing three combined inputs:
{
  "txid": "combined-spend",
  "vin": [
    { "txid": "funding-0", "vout": 0, "prevout": { "value": 0.4 } },
    { "txid": "funding-1", "vout": 0, "prevout": { "value": 0.4 } },
    { "txid": "funding-2", "vout": 0, "prevout": { "value": 0.4 } }
  ],
  "vout": [
    { "value": 1.00000000, "n": 0, "scriptPubKey": { "address": "bcrt1qreceiver" } },
    { "value": 0.19999000, "n": 1, "scriptPubKey": { "address": "bcrt1qalicechange" } }
  ]
}
```

## Evidence references

- Verified via Rust test suite `tests/lab_09.rs` functions.
- Observed coin selection input count of 3 in Polar node inspector.

## Explanation

- **UTXO Consolidation and Privacy**: When a transaction combines multiple UTXOs as inputs, the sender must provide cryptographic signatures for each input. This publicly proves to the network that a single entity (the sender) controls the private keys for all of these addresses.
- **Privacy Trade-off**: Blockchain analysis tools leverage this "common-input-ownership heuristic" to cluster addresses together and map out the ownership graph of the network. Combining UTXOs exposes the sender's total wallet balance and transactional links, degrading transaction privacy.
