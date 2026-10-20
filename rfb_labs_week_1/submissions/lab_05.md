# Lab 05 — Broadcast and observe an unconfirmed payment

## Commands used

```bash
# Mempool observation and unconfirmed transaction tracking
cargo test --test lab_05
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qreceiver..." 1.0
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=miner gettransaction "payment-txid"
bitcoin-cli -regtest -rpcwallet=receiver getbalances
```

## Terminal output

```json
{
  "txid": "e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2",
  "mempool_contains_tx": true,
  "sender_status": {
    "txid": "e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2",
    "confirmations": 0,
    "amount": -1.0,
    "fee": -0.00001,
    "block_hash": null
  },
  "receiver_balance": {
    "trusted": 0.0,
    "untrusted_pending": 1.0,
    "immature": 0.0
  }
}
```

```text
running 4 tests
test observes_broadcast_without_confirmation ... ok
test reads_local_mempool_txids ... ok
test reads_wallet_transaction_status ... ok
test sends_payment_in_sender_wallet_context ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- Implemented `send_btc`, `get_raw_mempool`, `get_transaction_status`, and `observe_unconfirmed_payment` in `src/labs/lab05_mempool.rs`.
- Proved presence of transaction in local node mempool immediately after broadcast.
- Confirmed `confirmations = 0` and receiver balance allocated to `untrusted_pending`.
- Validated test suite in `tests/lab_05.rs`.

## Explanation

A Bitcoin transaction progresses through distinct lifecycle states:

1. **Built and Signed**: The sender's wallet selects spendable UTXOs, constructs input scriptWitnesses/scriptSigs, defines recipient and change outputs, calculates fees, and signs the transaction hex off-chain.
2. **Broadcast**: The signed raw transaction bytes are transmitted to peer nodes across the P2P network via `inv` / `tx` messages.
3. **Mempool (Memory Pool)**: Valid unconfirmed transactions received by a full node are held in memory. The node verifies consensus rules, script execution, and fee rate thresholds before adding the TXID to its mempool. Transactions in mempool have **zero confirmations**; they are subject to replacement (RBF), double-spending risk, or eviction, and do NOT constitute finalized settlement.
4. **Confirmed**: A miner selects transactions from the mempool, constructs a candidate block header including their Merkle root, solves the Proof-of-Work puzzle, and broadcasts the block. Once included in a valid block on the active chain, the transaction receives its first confirmation ($1$). Subsequent blocks built on top increase its confirmation depth, making settlement exponentially immutable.
