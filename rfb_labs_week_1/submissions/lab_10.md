# Lab 10 — Competing branches and reorganization

## Commands used

Rust: `cargo run --example run10` (custom driver, second Bitcoin Core node `backend2` added in Polar):
- `getblockchaininfo` (both nodes) — common tip via `get_chain_tip`
- `getpeerinfo` / `disconnectnode` (both nodes) — sever the peer link
- `generatetoaddress 2 <address>` (node A) / `generatetoaddress 4 <address>` (node B) — private mining on each side
- `getblockchaininfo` (both nodes) — record competing tips
- `addnode "backend2"/"backend1" onetry` (both nodes) — reconnect
- `getblockchaininfo` (both nodes) — final tips after sync

## Terminal output

=== Lab 10: common tip before split ===
ChainTip {
height: 110,
best_block_hash: "206302c7fbf0cfaeaca89b3e54e98df8b3456099b4b646dea90c49e40dab0b66",
chainwork: "0000000000000000000000000000000000000000000000000000000000dde",
}

=== Disconnecting peers ===
node A: already had no peers
node B: already had no peers

=== Mining privately ===
node A private tip: ChainTip {
height: 112,
best_block_hash: "5a6c1d7600f37befb2fcf3a52c30ebda8a970d5e1389d1785a4c41e3b1a4081f",
chainwork: "00000000000000000000000000000000000000000000000000000000000e2",
}
node B private tip: ChainTip {
height: 114,
best_block_hash: "22486b8b3fa4195a59be128f00d20d7c413338c6193869aa6dfa9ec4b7efb490",
chainwork: "00000000000000000000000000000000000000000000000000000000000e6",
}

=== Reconnecting ===
node A final tip: ChainTip {
height: 114,
best_block_hash: "22486b8b3fa4195a59be128f00d20d7c413338c6193869aa6dfa9ec4b7efb490",
chainwork: "00000000000000000000000000000000000000000000000000000000000e6",
}
node B final tip: ChainTip {
height: 114,
best_block_hash: "22486b8b3fa4195a59be128f00d20d7c413338c6193869aa6dfa9ec4b7efb490",
chainwork: "00000000000000000000000000000000000000000000000000000000000e6",
}

=== Reorg report ===
ReorgReport {
common_tip_before_split: "206302c7fbf0cfaeaca89b3e54e98df8b3456099b4b646dea90c49e40dab0b66",
converged: true,
}

## Evidence references

Screenshot: `evidence/lab10.png`

## Explanation

Before splitting, both nodes agreed on one history: height 110, one shared best-block hash. Disconnecting them (I found both already had no active peers, confirming the split had taken effect) simulated a real network partition — the kind that can happen from routing issues or connectivity loss between geographically separated groups of nodes.

Once split, each node kept independently mining on top of the *same* last agreed block, producing two genuinely different, individually valid chains — a fork. I deliberately mined more blocks on node B (4) than node A (2), so node B's chain accumulated more total proof-of-work, visible directly in the `chainwork` field (`...e6` vs `...e2`).

On reconnecting, node A discarded its own two privately-mined blocks and adopted node B's four-block chain instead — a reorganization. This is exactly why `converged: true`: both nodes ended up on identical height, hash, and chainwork. Bitcoin Core doesn't decide which branch survives by which was mined first, which node has more reputation, or any social/identity signal — it purely compares accumulated chainwork and always adopts whichever valid chain has the most. Node A's abandoned blocks become "stale" — they were fully valid blocks, but they're no longer part of the chain everyone agrees is real. This is precisely why confirmation depth matters (Lab 08): a transaction buried under many blocks would require out-mining all of that accumulated work to reorg away, which becomes exponentially impractical the deeper it's buried.
