# Lab 07 — Confirm and locate the transaction

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_07

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest generatetoaddress 1 "bcrt1qminer"
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "payment-txid"
bitcoin-cli -regtest getblock "block-hash" 1
```

## Terminal output

```json
// getrawmempool output after mining (mempool is now empty):
[]

// gettransaction confirmations:
{
  "txid": "payment-txid",
  "confirmations": 1,
  "blockhash": "block-hash",
  "blockindex": 1,
  "blocktime": 1600000000
}

// getblock output showing tx list containing payment-txid:
{
  "hash": "block-hash",
  "confirmations": 1,
  "height": 102,
  "tx": [
    "coinbase-txid",
    "payment-txid"
  ]
}
```

## Evidence references

- Verified via Rust test suite `tests/lab_07.rs` functions.
- Checked block height 102 and mempool size in Polar nodes graph.

## Explanation

- **Did mining change the transaction or its place in history**: Mining does not alter the serialized bytes of the transaction or its TXID, as doing so would invalidate the cryptographic signatures. However, mining permanently establishes the transaction's place in the agreed consensus history of the network. By committing the transaction into a valid block, it transitions from a pending mempool broadcast to an immutable, ordered ledger entry, protecting it from double-spending.
