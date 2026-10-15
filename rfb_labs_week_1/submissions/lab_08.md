# Lab 08 — Block security

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). See `lab_01.md` for details.

## Commands used

```bash
# Verbose header of the block that confirmed the payment
bitcoin-cli -regtest -datadir=$LAB/node-a getblockheader <blockhash>

# Depth before
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=receiver gettransaction <txid>

# Mine five more blocks, then re-read the depth
bitcoin-cli -regtest -datadir=$LAB/node-a generatetoaddress 5 <mining-addr>
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=receiver gettransaction <txid>

# Rust implementation: lab08_security::{get_block_header, mine_additional_blocks,
# get_confirmations, build_security_report}
cargo test --test lab_08
cargo run --example week1_walkthrough
```

## Terminal output

```text
========== Lab 08 — headers, proof of work, and depth ==========
block hash      = 641d4e77a5c41aa867eac7dd6a2f62fb397ae2e19cba744aeee875cf0fad0c31
height          = 102
previous hash   = Some("72d44de2f35768f8645605c4555b545d37068c8ef1d94890c9af1956d171731c")
merkle root     = 3c3a43fc87b341c3767d474564062e5fe8d49ddd5ee1e07c21462b23d5ff96de
nonce           = 1
bits            = 207fffff
difficulty      = 0.00000000046565423739069247
confirmations   = 1
chainwork       = 00000000000000000000000000000000000000000000000000000000000000ce
depth before    = 1
depth after +5  = 6
```

The raw header, which also shows the `target` that `bits` encodes:

```json
{
  "hash": "641d4e77a5c41aa867eac7dd6a2f62fb397ae2e19cba744aeee875cf0fad0c31",
  "height": 102,
  "merkleroot": "3c3a43fc87b341c3767d474564062e5fe8d49ddd5ee1e07c21462b23d5ff96de",
  "nonce": 1,
  "bits": "207fffff",
  "target": "7fffff0000000000000000000000000000000000000000000000000000000000",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000ce",
  "nTx": 2,
  "previousblockhash": "72d44de2f35768f8645605c4555b545d37068c8ef1d94890c9af1956d171731c"
}
```

Every field the lab asks to record is present: block hash and height, previous-block
hash, Merkle root, nonce, `bits` together with its expanded `target` and `difficulty`,
confirmations, and accumulated chainwork. **One confirmation became six after mining
five blocks** — `depth before = 1`, `depth after +5 = 6`.

Two regtest-specific values are worth flagging honestly. `nonce = 1` and a `target` of
`7fffff00…` show that regtest's difficulty is set almost as low as possible: the miner
found a valid hash on roughly its second attempt. On mainnet the same header search takes
the entire network on the order of 10 minutes. The mechanism is identical; only the
target differs. Likewise `chainwork = 0xce` is 206 — the total work of 103 regtest
blocks, a number that on mainnet is astronomically larger.

## Evidence references

- Transcript section and raw `getblockheader` output above, both from the live run.
- Implementation: `src/labs/lab08_security.rs`. `build_security_report` reads the header
  and initial depth, mines five blocks, then re-reads the depth, so `confirmations_before`
  and `confirmations_after` bracket a known number of blocks.
- `previous_block_hash` is modelled as `Option<String>` because the genesis block is the
  one header with nothing behind it.
- `confirmations` is `i64`, not `u64`: a block on a stale branch reports `-1`, which
  Lab 10 relies on.
- Public tests: `cargo test --test lab_08` — 4 passed.
- No screenshots attached; the verbatim output above is the evidence.

## Explanation

**Hash links.** Each header contains `previousblockhash`, and each block's own hash is
computed over a header that includes that field. Block 102 names `72d44de2…` as its
parent, so changing anything in block 101 changes block 101's hash, which invalidates the
`previousblockhash` in block 102, which changes block 102's hash, and so on to the tip.
The chain is a linked list where every link is a cryptographic commitment: you cannot
edit history in place, you can only rebuild everything after the edit.

**Merkle commitment.** The header does not contain transactions — it contains one 32-byte
`merkleroot` (`3c3a43fc…`) that is the root of a binary hash tree over the block's
transaction list. Because it lives *inside* the hashed header, the block hash commits to
the exact set and order of transactions. Adding, removing, or altering any one of them
changes the root and therefore the block hash. It also has a practical payoff: proving a
transaction is in a block needs only the path from that transaction to the root — about
`log₂(n)` hashes — not the whole block, which is what makes light clients possible.

**Proof-of-work search.** `bits` encodes a `target`, and a block is valid only if its
header hash, read as a number, is below that target. Nothing in the header can be freely
chosen except the `nonce` (and, indirectly, the coinbase and timestamp), so mining is
brute force: change the nonce, rehash, check, repeat. Because the hash is unpredictable
there is no shortcut and no way to work backwards from a target hash to a nonce. Work is
therefore *proof of expenditure* — a valid header is evidence that a measurable amount of
computation was spent, and that evidence can be verified in a single hash operation.
Cheap to check, expensive to produce.

**Why confirmations raise the cost of reorganisation.** Each new block on top must itself
satisfy the target, so `chainwork` accumulates. To remove a transaction buried under six
blocks, an attacker must build an alternative branch from before the confirming block and
overtake the honest chain's accumulated work — redoing all six blocks *and* out-running
whatever the rest of the network mines in the meantime. Six confirmations is a convention,
not a rule: it is the depth at which that race is impractical for most attackers and most
amounts. Depth is a probabilistic guarantee, not a binary one, and the right depth scales
with the value at stake.

**Why depth cannot rescue an invalid transaction.** These are orthogonal properties. Every
node validates every transaction against the consensus rules independently — signatures,
that inputs exist and are unspent, that outputs do not exceed inputs, coinbase maturity,
and the rest. A block containing an invalid transaction is rejected outright, no matter
how much work sits on top of it; that work is simply wasted. Proof of work orders valid
history and makes reordering expensive. It never confers validity. This is precisely why
a longer chain built on an invalid block is not a threat: honest nodes never saw it as a
chain in the first place.
