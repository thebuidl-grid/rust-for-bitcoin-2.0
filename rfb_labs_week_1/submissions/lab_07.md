# Lab 07 — Confirmation and block membership

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). See `lab_01.md` for details.

## Commands used

```bash
# Mine exactly one block
bitcoin-cli -regtest -datadir=$LAB/node-a generatetoaddress 1 <mining-addr>

# Did the TXID leave the mempool?
bitcoin-cli -regtest -datadir=$LAB/node-a getrawmempool

# Depth and containing block, from the receiver's wallet
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=receiver gettransaction <txid>
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=receiver getbalances

# Ask the block itself which transactions it contains
bitcoin-cli -regtest -datadir=$LAB/node-a getblock <blockhash> 1

# Rust implementation: lab07_confirm::{mine_one_block, mempool_is_empty,
# transaction_confirmations, confirm_and_locate_transaction}
cargo test --test lab_07
cargo run --example week1_walkthrough
```

## Terminal output

```text
========== Lab 07 — confirmation and block membership ==========
txid                 = b031668a7932c09fc4b775fa8c56e45afcc6617bb14cb5233a44f20e1dcb36ee
mempool is empty     = true
confirmations        = 1
containing block     = 641d4e77a5c41aa867eac7dd6a2f62fb397ae2e19cba744aeee875cf0fad0c31
block contains txid  = true
receiver balances    = WalletBalances { trusted: 1.0, untrusted_pending: 0.0, immature: 0.0 }
```

`getblock` on the containing block, independent of any wallet:

```json
{
  "hash": "641d4e77a5c41aa867eac7dd6a2f62fb397ae2e19cba744aeee875cf0fad0c31",
  "height": 102,
  "tx": [
    "2df05681ee4b06f7e43ad8781d95627c9bf4a2b871e7c0ce3b65b73ee24b708e",
    "b031668a7932c09fc4b775fa8c56e45afcc6617bb14cb5233a44f20e1dcb36ee"
  ]
}
```

All five required claims:

- **The TXID left the mempool** — `mempool is empty = true`; `getrawmempool` returns `[]`.
- **The receiver's balance became trusted** — `trusted` moved from `0.0` (Lab 05) to
  `1.0`, and `untrusted_pending` fell from `1.0` to `0.0`. The same money, relabelled.
- **One confirmation** — `confirmations = 1`.
- **Bitcoin Core reports a containing block hash** — `641d4e77…`, where Lab 05 had
  `block_hash: None`.
- **That block's transaction list contains the TXID** — the `tx` array above holds two
  entries: the coinbase `2df05681…`, and our payment `b031668a…`.

The last point is the one worth being pedantic about. The wallet claiming a block hash
and the block itself listing the TXID are two separate assertions, and only the second is
independent of the wallet.

## Evidence references

- Transcript section quoted above, plus the raw `getblock` output, both from the live run.
- Implementation: `src/labs/lab07_confirm.rs`. `confirm_and_locate_transaction` reads the
  block hash and the confirmation count from a **single** `gettransaction` call, so the
  two facts necessarily describe the same observation rather than two moments that a
  concurrently mined block could have separated.
- It then verifies membership against `getblock`'s own `tx` array rather than trusting
  the wallet's `blockhash` field.
- Public tests: `cargo test --test lab_07` — 4 passed, including
  `proves_transaction_is_inside_confirming_block`.
- No screenshots attached; the verbatim output above is the evidence.

## Explanation

**Mining did not change the transaction.** Not one byte. The TXID before mining
(`b031668a…` in Lab 05) and after mining (`b031668a…` here) are identical, and that is
not a coincidence — the TXID *is* the hash of the serialized transaction. Any change to
its inputs, outputs, amounts, or signatures would produce a different TXID and a
different transaction. The bytes that were broadcast in Lab 05 are the exact bytes now
sitting in block 102.

**What changed is its place in the agreed history.** Before mining, the transaction was a
proposal held in one node's mempool: valid, but with nothing preventing it from being
replaced by a conflicting spend, evicted, or simply never included. After mining, a block
commits to it. The block's Merkle root is computed from its transaction list, so block
`641d4e77…` cannot contain a different set of transactions without becoming a different
block with a different hash — and that hash is what the next block builds on.

So confirmation does not make a transaction *more valid*; it was already fully valid.
Confirmation makes it **increasingly expensive to remove**. Undoing it now requires
producing a competing branch that excludes it and carries more accumulated work than the
current one. That is why the wallet's label flips from `untrusted_pending` to `trusted`:
nothing about the money changed, only the cost of reversing it.

This also explains the two other observations. The mempool emptied because the mempool
holds *candidates* for inclusion, and a transaction already in a block is no longer a
candidate. And the confirmation count is not a property stored in the transaction — it is
`current_height - block_height + 1`, recomputed every time you ask. It will rise by one
with every block, which is exactly what Lab 08 measures, and it can fall back to zero if
the containing block is ever orphaned, which is what Lab 10 demonstrates.
