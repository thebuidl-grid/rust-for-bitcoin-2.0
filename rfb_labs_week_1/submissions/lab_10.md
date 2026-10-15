# Lab 10 — Competing branches and reorganization

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). Node A listens on p2p 18444
> and node B on p2p 18455; both run on the same host. See `lab_01.md` for details.

## Commands used

```bash
# Link the two nodes and let them synchronise
bitcoin-cli -regtest -datadir=$LAB/node-a addnode 127.0.0.1:18455 onetry
bitcoin-cli -regtest -datadir=$LAB/node-a getblockchaininfo   # common tip
bitcoin-cli -regtest -datadir=$LAB/node-b getblockchaininfo

# Split the network
bitcoin-cli -regtest -datadir=$LAB/node-a disconnectnode 127.0.0.1:18455

# Mine privately on each side of the split
bitcoin-cli -regtest -datadir=$LAB/node-a generatetoaddress 2 <node-a-addr>
bitcoin-cli -regtest -datadir=$LAB/node-b generatetoaddress 4 <node-b-addr>

# Record both private tips and their chainwork
bitcoin-cli -regtest -datadir=$LAB/node-a getblockchaininfo
bitcoin-cli -regtest -datadir=$LAB/node-b getblockchaininfo

# Heal the split and wait for convergence
bitcoin-cli -regtest -datadir=$LAB/node-a addnode 127.0.0.1:18455 onetry
bitcoin-cli -regtest -datadir=$LAB/node-a getchaintips

# Rust implementation: lab10_reorg::{get_chain_tip, disconnect_peer,
# reconnect_peer, build_reorg_report}
cargo test --test lab_10
cargo run --example week1_walkthrough
```

A practical note: `disconnectnode` matches on the peer's listening address, and only the
node that *opened* the connection knows the peer by that address — the accepting side
sees an ephemeral port. Node A dials node B, so node A is also the node that must issue
the disconnect. Dropping the link from one side is sufficient; the two share a single TCP
connection, so asking node B to disconnect afterwards fails with
`Node not found in connected nodes`.

## Terminal output

```text
========== Lab 10 — competing branches and the most-work rule ==========
common height   = 109
common tip      = 608287d4b41cbeaf90892c35f8df764dcce823fc40d7ff69627a542a34a7670a
common chainwork= 00000000000000000000000000000000000000000000000000000000000000dc
nodes disconnected
node A private tip = 2f95eade2549fed3889cfafd386083dbbe82ea1efb1974ec46dba3a62d2d7128 at height 111 (chainwork 00000000000000000000000000000000000000000000000000000000000000e0)
node B private tip = 56fc2e683a240b5d48119b4acf0ff2ebacf1ec816307e28586b3e016ca56ac5a at height 113 (chainwork 00000000000000000000000000000000000000000000000000000000000000e4)
node A final tip   = 56fc2e683a240b5d48119b4acf0ff2ebacf1ec816307e28586b3e016ca56ac5a at height 113
node B final tip   = 56fc2e683a240b5d48119b4acf0ff2ebacf1ec816307e28586b3e016ca56ac5a at height 113
converged          = true
stale branch       = 2f95eade2549fed3889cfafd386083dbbe82ea1efb1974ec46dba3a62d2d7128 (node A's branch, 2 blocks above the common tip, discarded for node B's greater accumulated work)
```

**Chainwork is the deciding number.** Both branches start from `0xdc` at the common tip.
Node A's two blocks bring it to `0xe0` (+4); node B's four blocks bring it to `0xe4` (+8).
Node B's branch carries twice the work above the fork, and it wins.

**The reorganization, in node A's own `debug.log`** — the strongest single piece of
evidence here, because it shows node A *undoing* its own blocks:

```text
UpdateTip: new best=2f95eade…7128 height=111   <- node A's private tip
UpdateTip: new best=3681e78c…0a92 height=110   <- rewinding: block 111 disconnected
UpdateTip: new best=69c4d302…6a2e height=110   <- now on node B's branch instead
UpdateTip: new best=7b30ebfc…7da3 height=111
UpdateTip: new best=28fea578…e1bf height=112
UpdateTip: new best=56fc2e68…ac5a height=113   <- converged on node B's tip
```

Node A climbs to 111 on its own branch, walks back down to 110, switches to node B's
block at height 110, and then advances along B's branch to 113. Two blocks disconnected,
four connected.

**`getchaintips` on node A afterwards**, showing the abandoned branch still on disk:

```json
[
  { "height": 113, "hash": "56fc2e68…ac5a", "branchlen": 0, "status": "active" },
  { "height": 111, "hash": "2f95eade…7128", "branchlen": 2, "status": "valid-fork" }
]
```

`status: "valid-fork"` is precise and worth dwelling on: node A's two blocks were never
invalid. They were correctly constructed, correctly mined, and would have been accepted
had they won. They simply lost, and `branchlen: 2` records exactly how much was discarded.

## Evidence references

- Transcript section, `debug.log` extract, and `getchaintips` output above, all from the
  live run.
- Implementation: `src/labs/lab10_reorg.rs`. `build_reorg_report` requires **both** the
  final hashes and the final heights to match before reporting `converged`, so agreeing
  on a height while sitting on different blocks would not pass.
- `ChainTip` carries `chainwork` alongside height precisely because height is the wrong
  comparison; see the explanation below.
- The driver polls both nodes until their tips agree rather than sleeping a fixed
  interval, so `converged = true` reflects an observed state.
- Public tests: `cargo test --test lab_10` — 4 passed.
- No screenshots attached; the verbatim output above is the evidence.

## Explanation

**Why one branch became stale.** While the nodes were disconnected, each mined honestly on
top of the last block it knew about. Neither did anything wrong — they simply could not
see each other, so both extended block `608287d4…` at height 109. That produced two valid
chains sharing a common history and disagreeing about everything after the fork point.
This is not a failure mode; it is the normal consequence of a distributed system with
propagation delay, and it happens on mainnet whenever two miners find a block at nearly
the same time. When the nodes reconnected, one of the two branches had to be abandoned,
and node A's became **stale**: valid blocks that are not part of the chain anyone builds
on. The coinbase rewards in those two blocks simply cease to exist, which is the real
reason for the coinbase maturity rule from Lab 03.

**What a reorganization is.** Concretely, it is what the `debug.log` above shows: a node
rewinding its chain state to the fork point and rebuilding along a different branch. Node
A disconnected block 111, restoring the UTXO set to its state at height 110, then
connected node B's four blocks in order, validating each one. Any transaction that was in
node A's discarded blocks and not in node B's is returned to the mempool to be mined
again; its confirmation count drops from a positive number back to zero. This is why
confirmations can go *down*, and why "confirmed" is a matter of depth rather than a
binary state.

**Why the most-work rule, and not the alternatives.** Nodes choose the valid branch with
the greatest accumulated proof of work. The reason is that this is the only tiebreaker
that is *objective, verifiable, and expensive to influence* — a new node can download both
branches, verify every header, sum the work, and reach the same answer as everyone else
with no outside information and no trust in anybody.

Consider what the alternatives would mean:

- **Miner identity.** Requires knowing who miners are and agreeing on which of them to
  trust, which reintroduces exactly the central authority Bitcoin exists to remove.
  Identity is also free to fabricate — a Sybil attacker can claim to be a thousand miners.
- **Arrival time.** Unverifiable. Two nodes on opposite sides of the world genuinely
  receive competing blocks in different orders, so "first" is not a global fact. Anyone
  can also lie about timestamps.
- **Social claim.** Cannot be evaluated by software, and is trivially Sybil-attacked.

Accumulated work has none of these problems. It cannot be faked, because the only way to
produce a valid header is to actually search for one. It cannot be disputed, because any
node computes it from the headers alone. And it costs real resources, so overturning a
deep branch means outspending the honest network rather than merely out-arguing it.

Note precisely what the rule says: **the valid branch with the most work.** Validity is
checked first and independently. A branch containing an invalid transaction is not a
candidate at all, no matter how much work sits on it — as Lab 08 argues, work orders valid
history but never creates validity. And it is *most work*, not *longest*: in this lab the
two coincide because regtest difficulty is constant, but on a chain where difficulty
changes, a shorter branch of harder blocks legitimately beats a longer branch of easier
ones. Comparing heights would be the wrong test, which is why `ChainTip` records
chainwork.
