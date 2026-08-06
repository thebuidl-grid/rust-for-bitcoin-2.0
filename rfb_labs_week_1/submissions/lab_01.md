# Lab 01 — Regtest network inspection

## Commands used

```bash
cargo test --test lab_01
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash
```

## Terminal output

```text
Chain: [PASTE ACTUAL CHAIN]
Block height: [PASTE ACTUAL HEIGHT]
Best-block hash: [PASTE ACTUAL HASH]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- [Regtest chain, block height, and best-block hash](evidence/lab_01.png)

## Explanation

Polar manages the local Bitcoin test network through a graphical interface. Docker runs the Bitcoin Core node in an isolated container. Bitcoin Core validates transactions and blocks, stores the blockchain, and exposes the RPC methods used by the lab. Regtest is a private Bitcoin network where blocks can be generated on demand and the coins have no real-world value.

The `chain` field verifies that the node is using regtest. The block height identifies the current position in the chain, while the best-block hash identifies its current tip.
