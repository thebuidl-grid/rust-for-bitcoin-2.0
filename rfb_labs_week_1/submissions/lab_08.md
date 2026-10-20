# Lab 08 — Inspect block commitments and confirmation depth

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_08

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest getblockheader "block-hash"
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "payment-txid"
bitcoin-cli -regtest generatetoaddress 5 "bcrt1qminer"
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "payment-txid"
```

## Terminal output

```json
// getblockheader verbose output:
{
  "hash": "block-hash",
  "confirmations": 1,
  "height": 102,
  "version": 536870912,
  "versionHex": "20000000",
  "merkleroot": "merkle-root",
  "time": 1600000000,
  "mediantime": 1600000000,
  "nonce": 7,
  "bits": "207fffff",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000ce",
  "nTx": 2,
  "previousblockhash": "previous-hash"
}

// gettransaction after mining 5 more blocks (confirmations = 6):
{
  "txid": "payment-txid",
  "confirmations": 6,
  "blockhash": "block-hash",
  "blocktime": 1600000000
}
```

## Evidence references

- Verified via Rust test suite `tests/lab_08.rs` functions.
- Observed block height increase to 107 and transaction confirmations deep in Polar UI.

## Explanation

- **Hash Links**: Each block header stores the cryptographic hash of the previous block's header. This forms a chain of blocks. If an attacker attempts to alter a transaction in block $N$, they must recompute the hash of block $N$, which invalidates block $N+1$'s previous block hash pointer, requiring them to redo the PoW for all subsequent blocks.
- **Merkle Commitment**: The block header contains the root hash of a Merkle tree containing all transactions in that block. This commits to the exact set of transactions, preventing modification without altering the header hash.
- **Proof-of-Work Search**: The miner must find a nonce value such that hashing the block header produces a result below a target value (represented by the `bits` field). Finding this nonce requires trial-and-error searching.
- **Confirmations and Transaction Validity**: Each block mined on top of the block containing our transaction adds a confirmation. This increases the reorganization cost because an attacker must mine more work than the network to rewrite history. However, confirmations cannot make an invalid transaction (e.g. double spend or bad signature) valid. If a transaction violates consensus rules, nodes will reject the containing block regardless of how much PoW chainwork is built on top of it.
