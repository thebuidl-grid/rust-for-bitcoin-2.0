# Lab 07 — Confirmation and block membership

## Commands used

```bash
cargo test --test lab_07
bitcoin-cli -regtest generatetoaddress 1 "<MINER_ADDRESS>"
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=receiver getbalances
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<PAYMENT_TXID>"
bitcoin-cli -regtest getblock "<CONFIRMING_BLOCK_HASH>" 1
```

## Terminal output

```text
Mempool after mining: [PASTE ACTUAL EMPTY MEMPOOL]
Receiver trusted balance: [PASTE ACTUAL VALUE]
Transaction confirmations: [PASTE ACTUAL VALUE]
Confirming block hash: [PASTE ACTUAL HASH]
Block transaction list containing payment TXID: [PASTE RELEVANT OUTPUT]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- [Empty mempool and transaction with one confirmation](evidence/lab_07_a.png)
- [TXID included in the confirming block](evidence/lab_07_b.png)

## Explanation

Mining did not change the transaction's serialized contents or TXID. It changed the transaction's position in the network's agreed history: the transaction moved from a node's temporary mempool into an accepted block. The block hash identifies that containing block, and the block's transaction list proves membership.
