# Lab 10 — Competing branches and reorganization

## Commands used
- `bitcoin-cli -regtest getblockchaininfo` (both nodes) backs `get_chain_tip`
- `bitcoin-cli -regtest disconnectnode "127.0.0.1:18454"` / `addnode ... remove` initial
  disconnect attempt
- `bcli2 setnetworkactive false` backs `disconnect_peer` (a forceful, guaranteed isolation,
  used after a plain `disconnectnode` on localhost was immediately undone by Bitcoin Core's
  automatic peer reconnection)
- `bitcoin-cli -regtest generatetoaddress 2 <node1-address>`node 1 mines independently
- `bcli2 generatetoaddress 4 <node2-address>` (×2 batches, 8 blocks total) node 2 mines
  independently while isolated
- `bcli2 setnetworkactive true` + `bitcoin-cli -regtest addnode "127.0.0.1:18454" add` backs
  `reconnect_peer`
- `bitcoin-cli -regtest getblockchaininfo` (both nodes, post-reconnect) backs
  `build_reorg_report`

## Terminal output

### Common tip before split (both nodes synced, height 695)
$ bitcoin-cli -regtest getblockchaininfo
{
  "blocks": 695,
  "bestblockhash": "06be5a2e9afc3e60b25817ef8e4a9ab1dcbbc9880dd4ba561c71b240832597a2",
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000570"
}

# Isolate node 2 plain disconnectnode was auto-reconnected by Bitcoin Core's
# localhost peer logic, so setnetworkactive was used instead for a hard cut
$ bcli2 setnetworkactive false
false
$ bcli2 getpeerinfo
[]

# Node 1 mines 2 blocks in isolation
$ bitcoin-cli -regtest generatetoaddress 2 bcrt1qzaadqeq2ek5axlk7gyr8782y8ux85k8tz2uyqx
[
  "540e1bf84b85f14ee9ed4c88effaad1992372525ddfc5d9337cae969a6a59d5c",
  "348ca997d3e8a3f5adfaa3e3b369cb70e6aee05d5a7f2da8789760032f6ffd01"
]

# Node 2 mines 8 blocks total, in two batches of 4, while isolated
$ bcli2 generatetoaddress 4 bcrt1q8lgujcgh805v2hp32lhwch3mf9dfyggc2l42q0
[ "21e361c1...", "3445e722...", "1e3d584b...", "1b58dcf1..." ]
$ bcli2 generatetoaddress 4 bcrt1q8lgujcgh805v2hp32lhwch3mf9dfyggc2l42q0
[ "7095554a...", "25bd2dfe...", "68cb3d32...", "5fc9800315ac1bd4dd8970c2e087c72b25ba906631572c78a871de0d6e26fdd3" ]

# Competing tips while still isolated
$ bitcoin-cli -regtest getblockchaininfo
{
  "blocks": 703,
  "bestblockhash": "348ca997d3e8a3f5adfaa3e3b369cb70e6aee05d5a7f2da8789760032f6ffd01",
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000580"
}
$ bcli2 getblockchaininfo
{
  "blocks": 709,
  "bestblockhash": "5fc9800315ac1bd4dd8970c2e087c72b25ba906631572c78a871de0d6e26fdd3",
  "chainwork": "000000000000000000000000000000000000000000000000000000000000058c"
}

### Reconnect
$ bcli2 setnetworkactive true
true
$ bitcoin-cli -regtest addnode "127.0.0.1:18454" add

### Final tips after reconnection both nodes converged
$ bitcoin-cli -regtest getblockchaininfo
{
  "blocks": 709,
  "bestblockhash": "5fc9800315ac1bd4dd8970c2e087c72b25ba906631572c78a871de0d6e26fdd3",
  "chainwork": "000000000000000000000000000000000000000000000000000000000000058c"
}
$ bcli2 getblockchaininfo
{
  "blocks": 709,
  "bestblockhash": "5fc9800315ac1bd4dd8970c2e087c72b25ba906631572c78a871de0d6e26fdd3",
  "chainwork": "000000000000000000000000000000000000000000000000000000000000058c"
}

## Evidence references

Captured directly from two independent local Bitcoin Core regtest nodes: the
default node (datadir `~/.bitcoin`, RPC port 18443) and a second instance
(datadir `~/.bitcoin-regtest2`, RPC port 18453, P2P port 18454). Both started
from a common tip at height 695. While isolated, node 1 mined 2 blocks
(reaching height 703, chainwork `...580`) and node 2 mined 8 blocks (reaching
height 709, chainwork `...58c`) two genuinely competing chains. Upon
reconnection, node 1 discarded its own 2-block branch entirely and adopted
node 2's chain: both nodes converged on identical height (709), identical
bestblockhash, and identical chainwork, proving the higher-chainwork branch
won and the shorter branch was reorganized away.

Note: a plain `disconnectnode` call was insufficient to keep the two nodes
apart, since Bitcoin Core automatically re-established the manual/inbound
connection between them on localhost within moments of disconnecting.
`setnetworkactive false` was used on node 2 instead, which fully suspends all
networking until re-enabled, guaranteeing genuine isolation during
independent mining.

## Explanation (co-authored by Claude)

Bitcoin's consensus rule is not simply the longest chain wins measured by
block count it is the chain with the most accumulated proof of work wins,
measured by chainwork. In this experiment, node 1 mined only 2 blocks while
isolated, while node 2 mined 8. Even though both branches diverged from the
same common ancestor at height 695, node 2's branch represented significantly
more cumulative computational effort chainwork ...58c versus ...580 even
though the difficulty per block was identical on regtest.

When the two nodes reconnected, each compared its own chain against what the
other had to offer. Node 1, upon learning of node 2's higher-chainwork
branch, recognized its own 2-block branch as the weaker, or stale, one. It
discarded those 2 blocks entirely and adopted node 2's chain instead this
process is called a reorganization, or reorg. Concretely, node 1's
bestblockhash changed from its own 348ca997... to node 2's 5fc9800315...,
and its height jumped from 703 straight to 709, as if its own 2 blocks had
never been mined at all.

This experiment demonstrates something important about transaction
finality: any transaction that had only been confirmed inside those 2
discarded blocks would effectively become unconfirmed again after the reorg,
since those blocks are no longer part of the chain either node considers
canonical. This is precisely why more confirmations are treated as more
secure a transaction buried under many blocks would require an attacker to
out-mine the entire honest network across all of those blocks to ever get
reorganized away, which becomes exponentially harder the deeper it is
buried. A shallow, one-or-two-block fork like the one demonstrated here is
comparatively easy to produce and resolve, which is exactly why waiting for
only a single confirmation is not considered fully final for high-value
transactions.
