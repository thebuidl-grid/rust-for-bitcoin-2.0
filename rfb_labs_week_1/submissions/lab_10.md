# Lab 10 — Competing branches and reorganization

## Commands used

```bash
cargo test --test lab_10

# Run on both nodes before the split
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getpeerinfo

# Disconnect each node using the actual Polar P2P address
bitcoin-cli -regtest disconnectnode "<OTHER_NODE_P2P_ADDRESS>"

# Run on Node A
bitcoin-cli -regtest generatetoaddress 2 "<NODE_A_MINING_ADDRESS>"
bitcoin-cli -regtest getblockchaininfo

# Run on Node B
bitcoin-cli -regtest generatetoaddress 4 "<NODE_B_MINING_ADDRESS>"
bitcoin-cli -regtest getblockchaininfo

# Reconnect using the actual Polar P2P address
bitcoin-cli -regtest addnode "<OTHER_NODE_P2P_ADDRESS>" onetry

# Run on both nodes after synchronization
bitcoin-cli -regtest getblockchaininfo
```

## Terminal output

```text
Common height before split: [PASTE ACTUAL HEIGHT]
Common best-block hash: [PASTE ACTUAL HASH]
Node A private height/hash/chainwork: [PASTE ACTUAL VALUES]
Node B private height/hash/chainwork: [PASTE ACTUAL VALUES]
Node A final height/hash/chainwork: [PASTE ACTUAL VALUES]
Node B final height/hash/chainwork: [PASTE ACTUAL VALUES]
Converged on the same tip: [PASTE ACTUAL TRUE/FALSE]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- `submissions/evidence/lab_10_a.png` — planned screenshot showing the common tip and disconnected competing tips.
- `submissions/evidence/lab_10_b.png` — planned screenshot showing both nodes converging after reconnection.

## Explanation

While disconnected, the nodes extend different valid branches from the same parent. Node B's four-block branch accumulates more proof of work than Node A's two-block branch. After reconnection, Node A reorganizes from its former tip to the stronger valid branch, and its previous private blocks become stale. Nodes choose the valid chain with the greatest accumulated work, not the chain associated with a particular miner, arrival time, or social claim.
