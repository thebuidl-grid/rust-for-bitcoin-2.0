# Lab 08 — Block security

## Commands used

1. **Get verbose block header details**:
   ```bash
   bitcoin-cli getblockheader <blockhash>
   ```

2. **Mine 5 additional blocks**:
   ```bash
   bitcoin-cli -rpcwallet=miner generatetoaddress 5 <miner_address>
   ```

3. **Check final transaction confirmations**:
   ```bash
   bitcoin-cli -rpcwallet=receiver gettransaction <txid>
   ```

4. **Running tests**:
   ```bash
   cargo test --test lab_08
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_08` returns:
```text
running 4 tests
test reads_wallet_confirmation_depth ... ok
test mines_requested_confirmation_depth ... ok
test proves_one_confirmation_becomes_six ... ok
test decodes_proof_linked_block_header ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Sample block header fields (Mocked):
- `getblockheader` output:
  ```json
  {
    "hash": "block-hash",
    "confirmations": 1,
    "height": 102,
    "version": 536870912,
    "versionHex": "20000000",
    "merkleroot": "merkle-root",
    "time": 1722355500,
    "mediantime": 1722355450,
    "nonce": 7,
    "bits": "207fffff",
    "difficulty": 1e-8,
    "chainwork": "00000000000000ce",
    "previousblockhash": "previous-hash"
  }
  ```

---

## Evidence references

- Code is implemented in [lab08_security.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab08_security.rs).
- Unit tests pass successfully, showing correct retrieval of block headers, additional mining of confirmations, and reporting the transition from 1 to 6 confirmations.

---

## Explanation

- **Hash Links**: Each block header contains the hash of the preceding block (`previousblockhash`). This links all blocks together in an unbroken cryptographic chain back to the genesis block. If an attacker attempts to modify a transaction in a past block, its hash will change, causing a mismatch with the `previousblockhash` of the next block. To keep the chain valid, they would have to recompute the hashes (and recreate the Proof-of-Work) of every subsequent block.
- **Merkle Roots**: The `merkleroot` commits to all transactions included in the block. Transactions are hashed in pairs in a binary tree structure up to a single root hash. This root is committed inside the block header. This allows anyone to verify that a transaction is indeed in a block (using a compact Merkle proof) without needing to inspect the entire block.
- **Proof-of-Work**: The `nonce` is a value miners vary to find a block header hash that is less than the target difficulty represented by `bits`. The low target forces miners to try trillions of values (hashing power search) until a valid block is solved.
- **Confirmation Depth**: As more blocks are mined on top of the block containing our transaction, its confirmation depth increases. Each additional block adds proof-of-work security. Reorganizing the chain to double-spend a transaction with 6 confirmations requires mining 6 new blocks privately faster than the rest of the network, which is extremely expensive. Note that confirmations **do not** make an invalid transaction valid (consensus rules must still be followed), but they increase the economic cost of reorganizing history.
