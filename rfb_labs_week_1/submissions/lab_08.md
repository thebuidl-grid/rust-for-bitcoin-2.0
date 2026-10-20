# Lab 08 — Block security

## Commands used

Rust:

```
cargo test --test lab_08
cargo fmt --check
cargo run --example lab08
```

`examples/lab08.rs` calls the completed `get_block_header`, `mine_one_block`, and
`build_security_report` functions against the real node — sending a fresh payment, confirming it
once, then proving 1 confirmation becomes 6 after mining exactly 5 more blocks. Since the mining
here happens programmatically (one `generatetoaddress` call, count 5), the block count is exact.

Bitcoin Core RPCs (run directly in Polar's node terminal, cross-checking a separate payment from
Lab 07):

```
bitcoin-cli getblockheader <blockhash>
bitcoin-cli -rpcwallet=miner gettransaction <txid>
bitcoin-cli generatetoaddress 5 $MINER_ADDR
bitcoin-cli -rpcwallet=miner gettransaction <txid>
```

## Terminal output

`cargo test --test lab_08`:

```
running 4 tests
test mines_requested_confirmation_depth ... ok
test decodes_proof_linked_block_header ... ok
test reads_wallet_confirmation_depth ... ok
test proves_one_confirmation_becomes_six ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo run --example lab08` (real node, via the completed Rust implementation):

```
sent txid: 07dc3ebc0ec792d07a593778dab71cba24efd340bcdbe7606737b0714a3b83dd
confirmed in block: 4ce4dd1960e6668713ea01b34b11bebd7dcabf9d991ad31cac264319f8dae998
SecurityReport {
    header: BlockHeaderEvidence {
        hash: "4ce4dd1960e6668713ea01b34b11bebd7dcabf9d991ad31cac264319f8dae998",
        height: 217,
        previous_block_hash: Some("06548adefb20514eab7444d1945f0cdc1ff3115410d35a02744db5e43df70f6d"),
        merkle_root: "59a76de164417dc671e0ef656e5565b0617464d1abf6fae64ef13ab070cb653c",
        nonce: 0,
        difficulty: 4.6565423739069247e-10,
        bits: "207fffff",
        confirmations: 1,
        chainwork: "00000000000000000000000000000000000000000000000000000000000001b4",
    },
    confirmations_before: 1,
    confirmations_after: 6,
}
```

Raw `bitcoin-cli` cross-check, on a separate earlier payment (Lab 07's TXID,
`8ca3af42693c8e673adfe6646eb199773251ffac82246e6b71d49c0e06696dda`):

```
$ bitcoin-cli getblockheader 535b5cc5...
{ "hash": "535b5cc5...", "confirmations": 2, "height": 205, "nonce": 1, "bits": "207fffff",
  "merkleroot": "7e899ca3...", "chainwork": "...19c", "previousblockhash": "568c4f7b..." }

$ bitcoin-cli -rpcwallet=miner gettransaction 8ca3af42...
{ "confirmations": 2, "blockhash": "535b5cc5...", ... }

$ bitcoin-cli generatetoaddress 5 $MINER_ADDR
[ 5 block hashes ]

$ bitcoin-cli -rpcwallet=miner gettransaction 8ca3af42...
{ "confirmations": 12, ... }
```

Honest note on this second run: confirmations went from 2 to 12 (+10), not +5, even though
`generatetoaddress 5` was only run once. This matches a terminal double-send issue seen earlier in
this session (a command executing twice from a paste/history artifact) rather than a bug in the
mining logic — real block hashes were genuinely mined either way. The Rust example above avoids
this entirely, since it issues the `generatetoaddress` call programmatically exactly once, and
cleanly demonstrates the intended "1 becomes 6" proof.

## Evidence references

Terminal output above was captured directly from Polar's node terminal and from
`cargo run --example lab08`; no separate screenshots were taken for this lab.

## Explanation

- **Hash links**: every block header contains `previousblockhash`, the hash of the block directly
  before it. Since a block's own hash is computed over its header (which includes that previous
  hash), each block cryptographically commits to the entire chain of blocks before it — you can't
  change anything in an earlier block without changing its hash, which breaks every block built on
  top of it.
- **Merkle root**: a single hash that summarizes every transaction in the block, built by
  repeatedly hashing pairs of transaction hashes together up to one root. It's included in the
  header, so altering even one transaction in the block would change the Merkle root, which would
  change the header hash — the block would no longer match what every node already agreed to.
- **Proof-of-work search**: `nonce` (and implicitly the header's other fields) is varied by miners
  until the resulting block hash is numerically below the current `target` (derived from `bits`,
  e.g. `"207fffff"` here — regtest's minimum difficulty). Finding such a hash is expensive
  (effectively random search), which is what makes rewriting history costly: an attacker would have
  to redo that search for every block after the one they want to change, faster than the rest of
  the network extends the real chain.
- **Confirmations and reorg cost**: each additional block mined on top of a given block doesn't
  change anything about that block itself, but it *does* increase how much proof-of-work would need
  to be redone to replace it in a reorg — confirmations 1 → 6 means 5 more blocks' worth of work is
  now sitting on top, all of which an attacker would also have to out-race. Crucially, this doesn't
  make an *invalid* transaction valid — a transaction that broke consensus rules would be rejected
  by every honest node regardless of how many blocks tried to build on it. Confirmations only make a
  *valid* transaction's position in the chain progressively more expensive to undo.
