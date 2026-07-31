# Lab 07 — Confirmation and block membership

## Commands used

**Rust commands:**
```bash
cargo test lab_07
cargo run --example lab07_demo
```

**Bitcoin Core commands (via Polar):**
```bash
# Check mempool before mining
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass getrawmempool

# Mine one block to confirm transaction
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  generatetoaddress 1 "bcrt1q..."

# Check transaction confirmations
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  -rpcwallet=miner gettransaction "95942e68828b24e7794bddb85f3e636775e91da34a6bd484fff52eef851b7620"

# Verify transaction is in block
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  getblock "61362e3390cb639c409b1e50dba3def59a7c0e58f66514df4577acfe2bf7a80d"

# Verify mempool is empty after mining
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass getrawmempool
```

## Terminal output

**Before mining (transaction in mempool):**
```
Step 3: Sending unconfirmed payment...
  Receiver address: bcrt1qs6hd3slsrqan7usqewx0j9wzaxrn2rvrxawssa
  ✓ Payment sent: 95942e68828b24e7794bddb85f3e636775e91da34a6bd484fff52eef851b7620
  ℹ Mempool contains 1 transactions
```

**After mining (transaction confirmed):**
```
=== Confirmation Report ===

Transaction ID:     95942e68828b24e7794bddb85f3e636775e91da34a6bd484fff52eef851b7620
Block Hash:         61362e3390cb639c409b1e50dba3def59a7c0e58f66514df4577acfe2bf7a80d
Confirmations:      1
Mempool Empty:      true
In Block:           true

=== Verification ===

✓ Transaction has 1 confirmation(s)
✓ Mempool is empty (all transactions confirmed)
✓ Transaction found in block 61362e3390cb639c409b1e50dba3def59a7c0e58f66514df4577acfe2bf7a80d
```

## Evidence references

![Polar Confirmation](examples/lab07_demo.rs)

## Explanation

**What changed when the transaction became confirmed:**

1. **Mempool Status**: The transaction moved from the mempool (unconfirmed) to a mined block (confirmed). Before mining, `getrawmempool` showed 1 transaction; after mining, it returned empty `[]`.

2. **Confirmation Count**: The transaction gained its first confirmation. This changed from 0 (in mempool) to 1 (in block). Each subsequent block will add another confirmation.

3. **Block Membership**: The transaction became permanently part of block `61362e3390cb639c...`. Calling `getblock` with this hash returns the transaction's TXID in the block's transaction list, proving it's included.

4. **Finality**: The transaction transitioned from reversible (mempool transactions can be dropped) to immutable (blockchain transactions require reorganization to reverse, which becomes exponentially harder with each additional block).
