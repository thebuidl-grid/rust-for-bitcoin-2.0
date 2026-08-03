# Lab 07 — Confirmation and block membership

## Commands used

```bash
cargo test --test lab_07
```

RPC methods called:
- `generatetoaddress 1 <address>` - Mine exactly one block
- `getrawmempool` - List unconfirmed transactions
- `gettransaction <txid>` - Get transaction with confirmation count and block hash
- `getblock <hash> 1` - Get block with transaction list

## Terminal output

```
running 4 tests
test detects_empty_mempool ... ok
test mines_exactly_one_block ... ok
test proves_transaction_is_inside_confirming_block ... ok
test reads_confirmation_count ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, demonstrating:
- Mining creates blocks and clears mempool
- After mining, transaction has 1 confirmation and block_hash is set
- Transaction TXID appears in the block's tx array
- Mempool becomes empty after mining

## Explanation

Lab 07 demonstrates transaction confirmation - the critical transition from unconfirmed to confirmed:

1. **Confirmation Count**: 
   - **0 confirmations**: Transaction in mempool, not yet mined
   - **1 confirmation**: Included in latest block
   - **6+ confirmations**: Standard threshold for considering transaction "finalized"

2. **Block Membership**: Mining includes a transaction in a block. The block hash is the identifier of the block containing the transaction. Verifying the TXID appears in the block's tx array proves membership.

3. **Mempool Cleared**: After mining, the mempool empties (in regtest with a single node). In real networks, other transactions continue propagating.

4. **What Changes at Confirmation**:
   - Transaction moves from mempool to a block
   - `confirmations` field changes from 0 to 1
   - `blockhash` field is populated with the block's hash
   - Transaction output values move from `untrusted_pending` to `trusted` in receiver's balance

5. **Security Implication**: Each additional block on top increases confirmation depth, making reorganization exponentially more difficult. This is why services require 6 confirmations for settlement.
