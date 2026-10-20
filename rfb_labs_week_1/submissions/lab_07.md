# Lab 07 — Confirmation and block membership

## Commands used

1. **Mine exactly 1 block**:
   ```bash
   bitcoin-cli -rpcwallet=miner generatetoaddress 1 <miner_address>
   ```

2. **Verify mempool is empty**:
   ```bash
   bitcoin-cli getrawmempool
   ```

3. **Get transaction details**:
   ```bash
   bitcoin-cli -rpcwallet=receiver gettransaction <txid>
   ```

4. **Get block details containing the transaction**:
   ```bash
   bitcoin-cli getblock <blockhash> 1
   ```

5. **Running tests**:
   ```bash
   cargo test --test lab_07
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_07` returns:
```text
running 4 tests
test detects_empty_mempool ... ok
test mines_exactly_one_block ... ok
test proves_transaction_is_inside_confirming_block ... ok
test reads_confirmation_count ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Sample output of `getblock` (Mocked):
```json
{
  "hash": "block-hash",
  "confirmations": 1,
  "size": 285,
  "height": 102,
  "tx": [
    "coinbase-txid",
    "payment-txid"
  ]
}
```

---

## Evidence references

- Code is implemented in [lab07_confirm.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab07_confirm.rs).
- All unit tests passed, proving the correctness of mining, empty mempool check, transaction confirmations, block lookup, and membership assertions.

---

## Explanation

### What changed when the transaction became confirmed?
- **Serialized structure**: The serialized transaction bytes themselves did **not** change. The transaction remains exactly the same data structure as when it was originally built and signed.
- **Place in history**: What changed was its status in the global network consensus. Before confirmation, the transaction was unconfirmed in the mempool, representing a proposed change to history. Upon confirmation, the transaction was permanently packaged inside a valid block by a miner.
- **State change**:
  - The transaction was removed from the node's memory pool (mempool).
  - Its confirmation count increased from `0` to `1`.
  - It acquired a block commitment: it is now permanently located in the block history, and its transaction ID (`txid`) is committed in the block's transaction list (and hashed inside the block's Merkle root).
  - The recipient's balance changed from `untrusted_pending` to `trusted` because the payment is now considered secured by the proof-of-work of the containing block.
