# Lab 05 — Broadcast and mempool

## Commands used

```bash
cargo test --test lab_05
```

RPC methods called:
- `sendtoaddress <address> <amount>` - Create and broadcast a transaction
- `getrawmempool` - List all unconfirmed transactions in the mempool
- `gettransaction <txid>` - Get transaction details from wallet perspective
- `getbalances` - Inspect receiver's balance including pending funds

## Terminal output

```
running 4 tests
test observes_broadcast_without_confirmation ... ok
test reads_local_mempool_txids ... ok
test reads_wallet_transaction_status ... ok
test sends_payment_in_sender_wallet_context ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, demonstrating:
- Transactions broadcast to mempool with TXID returned immediately
- Mempool contains newly broadcast transactions
- Transaction shows 0 confirmations before mining
- Receiver shows `untrusted_pending` balance for unconfirmed funds

## Explanation

Lab 05 demonstrates the transaction lifecycle from broadcast to confirmation:

1. **Broadcasting**: `sendtoaddress` creates a signed transaction and broadcasts it to the network. The mempool (memory pool) stores unconfirmed transactions waiting for miners to include them in blocks.

2. **Transaction States**:
   - **Signed & Broadcast**: Transaction is created with valid signatures but not yet in any block
   - **Mempool**: Transaction accepted by nodes, waiting for confirmation
   - **0 Confirmations**: Transaction is broadcast but not yet mined
   - **Pending**: Receiver sees funds as `untrusted_pending` since they could be reversed via double-spend

3. **Fee Discovery**: Miners select transactions from the mempool based on fees. Higher fees incentivize faster inclusion. Without mining, transactions remain in mempool indefinitely.

4. **Double-Spend Protection**: Until a transaction reaches ~6 confirmations, it's vulnerable to double-spending if the chain reorganizes. This lab shows the critical window where transactions are broadcast but not yet permanently recorded.
