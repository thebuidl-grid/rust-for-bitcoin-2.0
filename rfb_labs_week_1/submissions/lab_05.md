# Lab 05 — Broadcast and mempool

## Commands used

1. **Send Bitcoin from miner wallet**:
   ```bash
   bitcoin-cli -rpcwallet=miner sendtoaddress <receiver_address> 1.0
   ```

2. **Retrieve the node's local mempool**:
   ```bash
   bitcoin-cli getrawmempool
   ```

3. **Get transaction status in miner wallet**:
   ```bash
   bitcoin-cli -rpcwallet=miner gettransaction <txid>
   ```

4. **Get receiver wallet balances**:
   ```bash
   bitcoin-cli -rpcwallet=receiver getbalances
   ```

5. **Running tests**:
   ```bash
   cargo test --test lab_05
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_05` returns:
```text
running 4 tests
test reads_local_mempool_txids ... ok
test reads_wallet_transaction_status ... ok
test observes_broadcast_without_confirmation ... ok
test sends_payment_in_sender_wallet_context ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Example terminal values (Mocked):
- `getrawmempool` output:
  ```json
  [
    "7ac16089307ff036c0a6b7d2db6b2e1f4094191bb819bc25501fb08cd3c13e51"
  ]
  ```
- `gettransaction` output:
  ```json
  {
    "txid": "7ac16089307ff036c0a6b7d2db6b2e1f4094191bb819bc25501fb08cd3c13e51",
    "amount": -1.00000000,
    "fee": -0.00001000,
    "confirmations": 0,
    "walletconflicts": []
  }
  ```
- `getbalances` output (receiver wallet):
  ```json
  {
    "mine": {
      "trusted": 0.00000000,
      "untrusted_pending": 1.00000000,
      "immature": 0.00000000
    }
  }
  ```

---

## Evidence references

- Code is implemented in [lab05_mempool.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab05_mempool.rs).
- All tests passed, proving the correctness of transaction broadcasting, mempool query, transaction confirmation tracking, and balance evaluation.

---

## Explanation

Here is the distinction between the four states of a transaction:

- **Signed (Built & Signed)**: The transaction is created locally and signed with private keys. It is valid, but it has not been shared with anyone yet. It only exists on the local machine/wallet.
- **Broadcast**: The signed transaction is sent out to the peer-to-peer network. Other nodes receive it and validate it against consensus rules.
- **Mempool**: If the broadcast transaction is valid, nodes store it in their local memory pool (mempool). It is now waiting to be picked up by a miner and included in a block. At this stage, it has **0 confirmations**, and the recipient sees it as "untrusted_pending" because it is not yet final in the blockchain history.
- **Confirmed**: A miner selects the transaction from the mempool and includes it in a new block. Once the block is solved and broadcast, the transaction has **1 confirmation**. As more blocks are mined on top, the confirmation count increases. The transaction is now part of the immutable blockchain consensus.
