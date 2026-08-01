# Lab 08 — Block security

## Commands used

```bash
cargo test --test lab_08
bitcoin-cli -regtest getblockheader <confirming-block-hash>
bitcoin-cli -regtest -rpcwallet=receiver gettransaction <payment-txid>
bitcoin-cli -regtest generatetoaddress 5 <miner-address>
bitcoin-cli -regtest -rpcwallet=receiver gettransaction <payment-txid>
```

## Terminal output

The header evidence included block hash, height, previous block hash, Merkle root, nonce, bits/difficulty, confirmations, and chainwork. Before mining more blocks the transaction had one confirmation; after five more blocks it had six confirmations.

## Evidence references

Evidence is the Lab 08 test run and the block-header transcript for the confirming block, plus the before/after receiver transaction confirmation counts.

## Explanation

Each block header links to the previous block hash, committing to chain order. The Merkle root commits to the transactions in the block. Proof of work is the search for a nonce/header hash meeting the target encoded by `bits`. More confirmations add more accumulated work above a transaction, increasing reorganization cost, but confirmations never make an invalid transaction valid.
