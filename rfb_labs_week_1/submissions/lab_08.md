# Lab 08 — Block security

## Commands used

```bash
cargo test --test lab_08
bitcoin-cli -regtest getblockheader "<CONFIRMING_BLOCK_HASH>"
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<PAYMENT_TXID>"
bitcoin-cli -regtest generatetoaddress 5 "<MINER_ADDRESS>"
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<PAYMENT_TXID>"
```

## Terminal output

```text
Block hash and height: [PASTE ACTUAL VALUES]
Previous-block hash: [PASTE ACTUAL VALUE]
Merkle root: [PASTE ACTUAL VALUE]
Nonce: [PASTE ACTUAL VALUE]
Bits/difficulty: [PASTE ACTUAL VALUES]
Initial confirmations: [PASTE ACTUAL VALUE]
Chainwork: [PASTE ACTUAL VALUE]
Confirmations after five additional blocks: [PASTE ACTUAL VALUE]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- [Confirming block header and transaction at one confirmation](evidence/lab_08_a.png)
- [Transaction after increasing from one to six confirmations](evidence/lab_08_b.png)

## Explanation

The previous-block hash links each block to its parent, while the Merkle root commits the header to the block's transaction set. Mining varies header data such as the nonce until the block hash satisfies the proof-of-work target represented by `bits`. Chainwork records accumulated proof of work. Additional confirmations place more proof-linked blocks above the payment, increasing the work required to reorganize it. Confirmations strengthen history but cannot make an invalid transaction valid.
