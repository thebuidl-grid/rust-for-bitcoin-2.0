# Lab 10 — Competing branches and reorganization

## Commands used

Rust:

```
cargo test --test lab_10
cargo fmt --check
cargo run --example lab10
```

`examples/lab10.rs` runs the full flow against two real Polar nodes (`backend1`/Node A,
`backend2`/Node B): reads the common tip, disconnects the peer, mines privately on both, reconnects,
and builds the final `ReorgReport`. Since Polar's nodes are configured to auto-reconnect almost
immediately, `setnetworkactive false`/`true` (plain RPC calls around the mining phase, not part of
the four graded functions) were needed to guarantee real isolation while each side mined — a plain
`disconnectnode` alone reconnected within the same second.

Bitcoin Core RPCs (run manually across both node terminals in Polar, before the Rust run):

```
# both nodes
bitcoin-cli getblockchaininfo

# backend1
bitcoin-cli getpeerinfo
bitcoin-cli disconnectnode <addr>
bitcoin-cli setnetworkactive false
bitcoin-cli generatetoaddress 2 <node-a-address>

# backend2
bitcoin-cli setnetworkactive false
bitcoin-cli createwallet nodeb
bitcoin-cli generatetoaddress 4 <node-b-address>

# both nodes
bitcoin-cli getblockchaininfo
bitcoin-cli setnetworkactive true

# backend1
bitcoin-cli addnode backend2:18444 onetry

# both nodes, after a moment
bitcoin-cli getblockchaininfo
```

## Terminal output

`cargo test --test lab_10`:

```
running 4 tests
test disconnects_peer_by_address ... ok
test reads_tip_and_accumulated_chainwork ... ok
test reconnects_peer_for_synchronization ... ok
test reports_convergence_on_the_stronger_branch ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo run --example lab10` (real two-node run, via the completed Rust implementation):

```
common tip before split: ChainTip {
    height: 239,
    best_block_hash: "16f7d82ccaa731fca541b019844bc1f2618f8f570b2f89546330bafd362fcc91",
    chainwork: "...01e0",
}
disconnect_peer called on A -> 172.18.0.3:41600
competing tips: ForkSnapshot {
    node_a: ChainTip { height: 241, best_block_hash: "24de7d85...", chainwork: "...01e4" },
    node_b: ChainTip { height: 243, best_block_hash: "14b06094...", chainwork: "...01e8" },
}
reconnect_peer called on A -> backend2:18444
final tips: ForkSnapshot {
    node_a: ChainTip { height: 243, best_block_hash: "14b06094...", chainwork: "...01e8" },
    node_b: ChainTip { height: 243, best_block_hash: "14b06094...", chainwork: "...01e8" },
}
ReorgReport {
    common_tip_before_split: "16f7d82c...",
    competing_tips: ForkSnapshot { node_a: .. height 241 .., node_b: .. height 243 .. },
    final_tips: ForkSnapshot { node_a: .. height 243, hash 14b06094 .., node_b: .. height 243, hash 14b06094 .. },
    converged: true,
}
```

Raw `bitcoin-cli` cross-check from the manual walkthrough (independent run, same behavior):

```
# before split, both nodes:
{ "blocks": 235, "bestblockhash": "119550e9...", "chainwork": "...01d8" }

# after mining privately:
backend1 (Node A, 2 blocks): { "blocks": 237, "bestblockhash": "7b162d57...", "chainwork": "...01dc" }
backend2 (Node B, 4 blocks): { "blocks": 239, "bestblockhash": "16f7d82c...", "chainwork": "...01e0" }

# after reconnecting, both nodes:
{ "blocks": 239, "bestblockhash": "16f7d82c...", "chainwork": "...01e0" }
```

In the manual run, Node A converged onto Node B's chain — matching the Rust run's outcome (the node
with less accumulated work always adopts the stronger branch). Node A's own 2 privately-mined
blocks in that run became orphaned/stale, discarded once the stronger chain arrived.

## Evidence references

Terminal output above was captured directly from both Polar node terminals and from
`cargo run --example lab10`; no separate screenshots were taken for this lab.

## Explanation

While disconnected, Node A and Node B each kept extending their own local view of "the chain" with
valid, honestly-mined blocks — from either node's own perspective, its private chain was completely
valid. This is exactly what a network partition looks like: no rule was broken, they simply
couldn't see each other's new blocks yet.

Once reconnected, each node compares total **accumulated proof-of-work** (`chainwork` — not block
count, not which chain arrived "first," not any notion of miner identity) across every chain it
knows about, and adopts whichever has the most. Here, Node B's 4 blocks represented more
accumulated work than Node A's 2 blocks, so Node A discarded its own blocks and rebuilt its active
chain on top of Node B's — a **reorganization**. The blocks Node A gave up are now "stale" (or
"orphaned"): they were real, validly-mined blocks that simply lost the competition for which
history the network settles on.

Nodes don't pick a winner by miner identity, arrival time, or any social/reputational signal — only
accumulated work, because that's the one property that's expensive to fake. This is also precisely
why coinbase rewards need 100 confirmations before they're spendable (Lab 03): the deeper a block
sits under subsequent blocks, the more accumulated work a competing branch would need to catch up
and overtake it, making a reorg that erases it exponentially more expensive the longer it's been
confirmed.
