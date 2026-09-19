# Lab 01 — Regtest network inspection

## Commands used

```bash
cd rfb_labs_week_1
cargo test --test lab_01
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash
```

## Terminal output

The Rust implementation passed all Lab 01 public checks. The node inspection commands returned `chain=regtest`, a numeric block height, and the current best-block hash, matching the `NetworkSnapshot { chain, block_height, best_block_hash }` fields.

## Evidence references

Evidence is the Lab 01 test run plus the Bitcoin Core RPC transcript from the Polar node named `Week 1 Bitcoin Fundamentals`. The recorded fields are chain name, height, and best-block hash.

## Explanation

Polar is the local UI/orchestrator used to create and manage the regtest Bitcoin network. Docker runs the Bitcoin Core node as an isolated container. Bitcoin Core is the node software that validates blocks, maintains the chainstate, exposes RPCs, and owns the mempool. Regtest is a private Bitcoin network mode where blocks are mined on demand, making it ideal for repeatable labs with fake coins.
