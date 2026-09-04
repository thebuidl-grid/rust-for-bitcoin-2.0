# Lab 05 — Broadcast and mempool

## Commands used

```bash
cargo test --test lab_05
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<RECEIVER_ADDRESS>" 1
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=miner gettransaction "<PAYMENT_TXID>"
bitcoin-cli -regtest -rpcwallet=receiver getbalances
```

## Terminal output

```text
Payment TXID: [PASTE ACTUAL TXID]
Mempool entry: [PASTE OUTPUT CONTAINING TXID]
Sender confirmations: [PASTE ACTUAL VALUE]
Sender amount and fee: [PASTE ACTUAL VALUES]
Receiver trusted balance: [PASTE ACTUAL VALUE]
Receiver untrusted-pending balance: [PASTE ACTUAL VALUE]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- [TXID in the mempool, zero confirmations, and receiver pending balance](evidence/lab_05.png)

## Explanation

The sender's wallet first builds and signs the transaction. Broadcasting announces it to connected nodes. A node that validates and accepts it stores it in its local mempool, but this does not make it part of the blockchain. Confirmation occurs only after a miner includes the transaction in an accepted block. Therefore, broadcast and mempool acceptance are not the same as confirmation.
