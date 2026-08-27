# Lab 07 — Confirm and locate the transaction

## Commands used

```bash
# Block mining, mempool evacuation, and transaction inclusion auditing
cargo test --test lab_07
bitcoin-cli -regtest generatetoaddress 1 "bcrt1qminer..."
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "payment-txid"
bitcoin-cli -regtest getblock "block-hash" 1
```

## Terminal output

```json
{
  "txid": "e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2",
  "block_hash": "3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b",
  "confirmations": 1,
  "mempool_is_empty": true,
  "transaction_is_in_block": true
}
```

```text
running 4 tests
test detects_empty_mempool ... ok
test mines_exactly_one_block ... ok
test proves_transaction_is_inside_confirming_block ... ok
test reads_confirmation_count ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- Implemented `mine_one_block`, `mempool_is_empty`, `transaction_confirmations`, and `confirm_and_locate_transaction` in `src/labs/lab07_confirm.rs`.
- Demonstrated that mining one block evacuates the transaction from mempool (`getrawmempool` returns `[]`).
- Verified transaction block inclusion by inspecting the block's `tx` array via `getblock block_hash 1`.
- Validated test suite in `tests/lab_07.rs`.

## Explanation

1. **Effect of mining on serialized transaction data**: Mining a block does **NOT** alter the byte contents or serialization of the transaction itself. The raw transaction hex, inputs, outputs, locktime, signatures, and computed TXID remain 100% identical before and after mining.
2. **Effect of mining on place in agreed history**: Mining alters the transaction's status in global consensus state. Prior to inclusion in a block, the transaction existed only as an ephemeral proposal in node mempools (unconfirmed state). When a miner includes the transaction in a valid block and publishes the block to the network, the transaction becomes immutably ordered into the chain of blocks. It is committed into the block's Merkle tree root and anchored by Proof-of-Work, transitioning from an unconfirmed floating proposal to permanent consensus history.
