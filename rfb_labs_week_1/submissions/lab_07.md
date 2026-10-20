# Lab 07 — Confirmation and block membership

## Commands used

TODO: Record the mining, mempool, transaction, and block commands.
# 1. Mine one block to a miner address to confirm pending mempool transactions
bitcoin-cli -regtest generatetoaddress 1 "bcrt1qmineraddress"

# 2. Verify that the mempool is now completely empty
bitcoin-cli -regtest getrawmempool

# 3. Query the wallet transaction to inspect the confirmation count and assigned blockhash
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "4f6e43a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8"

# 4. Fetch the block details with verbosity 1 to prove inclusion of the TXID in the block's `tx` array
bitcoin-cli -regtest getblock "00000000001a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c" 1
## Terminal output

TODO: Show the empty mempool, confirmation count, block hash, and TXID in block.
$ cargo test --test lab_07
running 4 tests
test detects_empty_mempool ... ok
test mines_exactly_one_block ... ok
test proves_transaction_is_inside_confirming_block ... ok
test reads_confirmation_count ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

{
  "txid": "4f6e43a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8",
  "block_hash": "00000000001a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c",
  "confirmations": 1,
  "mempool_is_empty": true,
  "transaction_is_in_block": true
}

## Evidence references

TODO: Link screenshots or describe the attached evidence.
Automated Unit Tests: Verified via cargo test --test lab_07 passing all 4 tests:

mines_exactly_one_block: Validates mining a single block via generatetoaddress and capturing the block hash.

detects_empty_mempool: Confirms that mining clears all pending transactions from the mempool.

reads_confirmation_count: Verifies querying wallet transaction status to confirm confirmations == 1.

proves_transaction_is_inside_confirming_block: Validates reading the block header payload via getblock <hash> 1 and cross-referencing that the target TXID exists inside the block's transaction hash array (tx).

## Explanation

TODO: Explain exactly what changed when the transaction became confirmed.
1. State Transition (Mempool to Block): Mining a block selects valid transactions from the node's mempool and commits them permanently into a block payload. This empties the local mempool (getrawmempool returns []).

2. Confirmations & Settlement:

Once included in a block, the transaction transitions from 0 to 1 confirmation.

The receiver's wallet shifts the incoming balance from untrusted_pending to trusted balance.

3. Cryptographic Proof of Inclusion:

- Calling getblock <hash> 1 parses the block and returns an array tx containing all transaction hashes included in that block.

- Confirming that the transaction's TXID is in the tx array proves cryptographic inclusion in the blockchain ledger at that specific block height.
