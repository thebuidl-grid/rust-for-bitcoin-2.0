# Lab 03 — Coinbase maturity

## Commands used

```bash
cargo test --test lab_03
bitcoin-cli -regtest generatetoaddress 1 "<MINER_ADDRESS>"
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest -rpcwallet=miner getbalances
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<RECEIVER_ADDRESS>" 1
bitcoin-cli -regtest generatetoaddress 100 "<MINER_ADDRESS>"
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest -rpcwallet=miner getbalances
```

## Terminal output

```text
Height after first block: [PASTE ACTUAL HEIGHT]
Balance after first block: [PASTE ACTUAL GETBALANCES OUTPUT]
Premature-spend error: [PASTE ACTUAL ERROR]
Final height: [PASTE ACTUAL HEIGHT]
Final balance: [PASTE ACTUAL GETBALANCES OUTPUT]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- [First block and its 50 BTC immature reward](evidence/lab_03_a.png)
- [Failed attempt to spend the immature reward](evidence/lab_03_b.png)
- [Height 101 and the matured first reward](evidence/lab_03_c.png)

## Explanation

A coinbase transaction creates the miner's block reward, but consensus prevents that output from being spent immediately. On a fresh chain, the first reward is created at height 1. After 100 additional blocks, the chain reaches height 101 and the first reward has sufficient maturity to become trusted and spendable. The later rewards remain immature because each was created at a more recent height.
