# Lab 07 — Confirmation and block membership

## Commands used

```
cargo test --test lab_07
cargo fmt --check
BITCOIN_CLI=<bitcoin-cli wrapper> cargo run --example lab07_demo
```

Underlying RPCs (`src/labs/lab07_confirm.rs`):
```
generatetoaddress 1 <address>          # confirming block (mined while gathering Lab 06 evidence)
getrawmempool
gettransaction cfb0ea59...29f1cfe      -rpcwallet=receiver
getblock 0e0e0b599c631219e78abae3a7c965c07117cb5943e90cc6bdf72df803c38c58 1
getbalances                            -rpcwallet=receiver   # cross-check only
```

Note: the single confirming block for this transaction was mined during Lab
06 evidence-gathering (see `submissions/lab_06.md` — needed to make
`getrawtransaction`'s `prevout` field populate on this Bitcoin Core build), so
this demo calls the granular Lab 07 functions (`mempool_is_empty`,
`transaction_confirmations`) plus a direct `getblock` membership check against
that already-confirmed state, rather than re-running
`confirm_and_locate_transaction` (which mines its own block and would push
the transaction to 2 confirmations instead of exactly 1).
`confirm_and_locate_transaction`'s end-to-end logic is verified against mocks
by `cargo test --test lab_07`.

## Terminal output

`cargo test --test lab_07`:
```
running 4 tests
test detects_empty_mempool ... ok
test reads_confirmation_count ... ok
test mines_exactly_one_block ... ok
test proves_transaction_is_inside_confirming_block ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab07_demo` against the live node:
```
mempool_is_empty = true
confirmations    = 1
block_hash       = 0e0e0b599c631219e78abae3a7c965c07117cb5943e90cc6bdf72df803c38c58
block contains txid? = true
```

Cross-check: `bitcoin-cli -rpcwallet=receiver getbalances` →
`{ "trusted": 1.0, "untrusted_pending": 0.0, "immature": 0.0 }`.

## Evidence references

- Screenshot: `submissions/images/Screenshot from 2026-08-01 13-58-25.png` — IDE
  terminal running `cargo test --test lab_07`, all 4 tests passing.
- `getrawmempool` returns `[]` — TXID `cfb0ea59...29f1cfe` has left the
  mempool.
- The receiver's `getbalances` moved from Lab 05's
  `untrusted_pending: 1.0, trusted: 0.0` to `trusted: 1.0,
  untrusted_pending: 0.0` — the payment is now trusted, spendable funds.
- `gettransaction` (receiver context) reports `confirmations: 1`.
- Bitcoin Core reports containing block hash
  `0e0e0b599c631219e78abae3a7c965c07117cb5943e90cc6bdf72df803c38c58`, and that
  block's `tx` array (via `getblock ... 1`) contains the TXID — confirmed
  membership.

## Explanation

Mining the block changed none of the previously signed and broadcast bytes of
the transaction, and it did not change its place in the *agreed* history
either, in the sense that the transaction's ordering relative to its own
inputs was already fixed the moment it was validated and accepted into every
node's mempool — mining didn't reorder anything about the transaction itself.

What mining *did* change is the transaction's status from provisional to
committed: before this block, the transaction existed only in each node's
private, transient mempool view — a view that isn't shared consensus and could
differ node to node, or vanish. Once the transaction is included in a mined
block that other nodes accept as the best chain, its position becomes part of
the chain's serialized, hash-linked, globally agreed history — every full node
that has this block, in this position, sees the exact same set of
transactions in the exact same order. That's the real change: from "probably
will happen" (mempool) to "cryptographically fixed as having happened"
(confirmed), which is exactly why the receiver's balance flips from
`untrusted_pending` to `trusted` at this moment and not before.
