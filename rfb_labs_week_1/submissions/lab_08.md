# Lab 08 — Block security

## Commands used

```
cargo test --test lab_08
cargo fmt --check
BITCOIN_CLI=<bitcoin-cli wrapper> cargo run --example lab08_demo
```

Underlying RPCs (`src/labs/lab08_security.rs`):
```
getblockheader 0e0e0b599c631219e78abae3a7c965c07117cb5943e90cc6bdf72df803c38c58
gettransaction cfb0ea59...29f1cfe   -rpcwallet=receiver   # confirmations, before
generatetoaddress 5 bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv
gettransaction cfb0ea59...29f1cfe   -rpcwallet=receiver   # confirmations, after
```

## Terminal output

`cargo test --test lab_08`:
```
running 4 tests
test decodes_proof_linked_block_header ... ok
test mines_requested_confirmation_depth ... ok
test proves_one_confirmation_becomes_six ... ok
test reads_wallet_confirmation_depth ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab08_demo` against the live node:
```
SecurityReport {
    header: BlockHeaderEvidence {
        hash: "0e0e0b599c631219e78abae3a7c965c07117cb5943e90cc6bdf72df803c38c58",
        height: 104,
        previous_block_hash: Some("6342ab2c2cc73fc79f4f45890026bfcc35ec82bd9f84450570d1f4cf1ea2b891"),
        merkle_root: "7b2197c51adff89f7f089444b90c5318af63716c6dcbd3a5301fe4b6d3fc085b",
        nonce: 1,
        difficulty: 4.6565423739069247e-10,
        bits: "207fffff",
        confirmations: 1,
        chainwork: "00000000000000000000000000000000000000000000000000000000000000d2",
    },
    confirmations_before: 1,
    confirmations_after: 6,
}
```

## Evidence references

- Screenshot: `submissions/evidence/Screenshot from 2026-08-01 13-58-42.png` — IDE
  terminal running `cargo test --test lab_08`, all 4 tests passing.
- Block hash `0e0e0b599c631219e78abae3a7c965c07117cb5943e90cc6bdf72df803c38c58`
  at height `104`.
- `previous_block_hash` = `6342ab2c2cc73fc79f4f45890026bfcc35ec82bd9f84450570d1f4cf1ea2b891`,
  linking this block to its parent.
- `merkle_root` = `7b2197c51adff89f7f089444b90c5318af63716c6dcbd3a5301fe4b6d3fc085b`.
- `nonce` = `1`, `bits` = `207fffff` (regtest's near-zero minimum difficulty
  target), `difficulty` ≈ `4.66e-10`, `chainwork` =
  `00000000000000000000000000000000000000000000000000000000000000d2`.
- `confirmations_before = 1` → after mining 5 more blocks,
  `confirmations_after = 6`, directly proving the payment transaction's
  confirmation depth grew by exactly the number of blocks mined on top of it.

## Explanation

**Hash links**: each block header stores the hash of the *previous* block
(`previous_block_hash`), so blocks form a backward-pointing chain — you cannot
change any earlier block without changing its hash, which breaks every
subsequent block's link to it (and all their proof-of-work becomes invalid),
making tampering with history detectable and, deeper in the chain,
prohibitively expensive to redo.

**Merkle commitment**: `merkle_root` is a single hash that summarizes every
transaction in the block via a binary hash tree. Anyone can verify that a
specific transaction is included in the block using only that transaction, a
short Merkle proof, and the root — without needing every transaction in the
block. Changing any transaction inside the block changes the Merkle root,
which changes the block hash, which breaks the hash-link chain described
above.

**Proof-of-work search**: miners repeatedly vary the `nonce` (and other
fields) and hash the header, searching for a hash at or below the `bits`
target. On regtest, `bits = 207fffff` sets the minimum-difficulty target so
this search succeeds almost immediately (`nonce = 1` here), unlike mainnet
where it takes enormous computation — but the mechanism is identical: finding
a valid header requires real, unfakeable work.

**Confirmations vs. validity**: confirmations count how many blocks have been
built on top of the block containing a transaction. Confirmations do not
retroactively make an *invalid* transaction valid — validity (correct
signatures, no double-spend, script rules, etc.) is checked once, when the
transaction is accepted into a block, and never changes afterward. What
increasing confirmations *does* do is raise the cost of a reorg deep enough to
undo that block: an attacker would need to rebuild not just that block but
every block mined on top of it with more accumulated proof-of-work than the
honest chain, which grows exponentially harder with each additional
confirmation. Six confirmations is the traditional "safe" depth precisely
because re-doing six blocks' worth of work is already impractical for all but
the most powerful adversaries.
