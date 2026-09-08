# Lab 10 — Competing branches and reorganization

## Commands used

A second Bitcoin Core node was added by hand (Polar's single-node network
only had one node, so a second `bitcoind` container of the same image was
started on the same Docker network and pointed at the first):
```
docker run -d --name polar-n2-backend2 --network polar-network-1_default \
  polarlightning/bitcoind:30.0 bitcoind -server=1 -regtest=1 ... -txindex=1 ...
bitcoin-cli (node B) addnode "polar-n1-backend1:18444" "add"   # initial sync
```

Rust commands:
```
cargo test --test lab_10
cargo fmt --check
BITCOIN_CLI_A=<node A wrapper> BITCOIN_CLI_B=<node B wrapper> \
  cargo run --example lab10_report_demo
```

Underlying RPCs (`src/labs/lab10_reorg.rs`), run against both nodes:
```
getblockchaininfo                       # get_chain_tip, both nodes -> common tip
addnode "polar-n1-backend1:18444" "remove"   # drop the persistent connection
disconnectnode "polar-n1-backend1:18444"     # disconnect_peer
generatetoaddress 2 <node A address>         # private mining on node A
generatetoaddress 4 <node B address>         # private mining on node B
getblockchaininfo                       # get_chain_tip, both nodes -> competing tips
addnode "polar-n1-backend1:18444" "onetry"   # reconnect_peer
getblockchaininfo                       # get_chain_tip, both nodes -> final tips
```

**A real-node finding worth recording:** the first attempt called
`disconnect_peer` (which runs plain `disconnectnode`) without first removing
the peer via `addnode ... remove`. Because the initial sync had used
`addnode ... add` (a *persistent* connection request), Bitcoin Core
transparently redialed and reconnected the two nodes within moments of the
disconnect — so both "private" chains were actually mined while the nodes
were still relaying blocks to each other in real time, and they converged
trivially because there was never a real fork (confirmed by `getpeerinfo`
still showing 1 connected peer moments after `disconnectnode`, and both nodes
finishing at the *identical* height/hash instead of a divergent one). The fix
was to `addnode ... remove` the persistent entry before `disconnectnode`,
which produced a real, verified split (`getpeerinfo` returned zero peers on
both sides even after a 5-second wait) before mining diverging chains.

## Terminal output

`cargo test --test lab_10`:
```
running 4 tests
test reads_tip_and_accumulated_chainwork ... ok
test disconnects_peer_by_address ... ok
test reconnects_peer_for_synchronization ... ok
test reports_convergence_on_the_stronger_branch ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Common tip before the (real) split, confirmed identical on both nodes:
```
height 118, hash 2311a984f93770e7fce928fb8b5fb11d65405e539f9defd4950a9c674e1585ed
```

After removing the persistent peer and disconnecting, `getpeerinfo` returned
`0` peers on both node A and node B (checked immediately and again after a
5-second wait, ruling out auto-reconnect).

Private mining while disconnected — node A mined 2 blocks, node B mined 4:
```
node A private tip: height 120, hash 101674d0264507d0ad5038fa4310e25e88f80a92215cbfa03b63eed24082e5bf,
                     chainwork ...0000f2
node B private tip: height 122, hash 173bddc09617c0412d24c58ccaa8be5c46047f38d99bc04633ea17c1cbeac643,
                     chainwork ...0000f6
```

`cargo run --example lab10_report_demo` (live `get_chain_tip` calls against
both nodes after reconnecting via `addnode ... onetry`, feeding the recorded
common/competing tips above into `build_reorg_report`):
```
live final tip (node A) = ChainTip { height: 122, best_block_hash: "173bddc0...eac643", chainwork: "...0000f6" }
live final tip (node B) = ChainTip { height: 122, best_block_hash: "173bddc0...eac643", chainwork: "...0000f6" }

ReorgReport {
    common_tip_before_split: "2311a984...e1585ed",
    competing_tips: ForkSnapshot {
        node_a: ChainTip { height: 120, best_block_hash: "101674d0...082e5bf", chainwork: "...0000f2" },
        node_b: ChainTip { height: 122, best_block_hash: "173bddc0...eac643",  chainwork: "...0000f6" },
    },
    final_tips: ForkSnapshot {
        node_a: ChainTip { height: 122, best_block_hash: "173bddc0...eac643", chainwork: "...0000f6" },
        node_b: ChainTip { height: 122, best_block_hash: "173bddc0...eac643", chainwork: "...0000f6" },
    },
    converged: true,
}
```

## Evidence references

- Screenshot: `submissions/evidence/Screenshot from 2026-08-01 13-58-42.png` — IDE
  terminal running `cargo test --test lab_10`, all 4 tests passing.
- Both nodes started from the identical common tip (height 118,
  `2311a984...`), proven by matching `getblockchaininfo` output on both
  before the split.
- After a verified disconnect (`getpeerinfo` = 0 peers, both sides, stable
  over 5 seconds), node A privately mined to height 120 and node B privately
  mined to height 122 — two genuinely different tips, with node B's chain
  carrying strictly more accumulated `chainwork` (`...f6` > `...f2`).
- After reconnecting via a one-time `addnode ... onetry`, both nodes'
  `getblockchaininfo` converge on height 122, hash `173bddc0...` — node B's
  chain. `build_reorg_report` computes `converged: true` from exactly this
  data (`final_tips.node_a == final_tips.node_b`).
- Node A's two independently-mined blocks (heights 119–120 on its private
  branch) are absent from the final agreed chain — they became stale/orphaned
  the moment node A adopted node B's longer chain.

## Explanation

Node A's 2-block branch became **stale** because, once reconnected, node A's
validation rules compared the two competing valid chains by total accumulated
proof-of-work (`chainwork`) rather than length, recency, or any notion of
"whose turn it was" — and node B's 4-block branch had strictly more chainwork
than node A's 2-block branch (both chains extend the same 118-block common
history, so more blocks here also means more work). Bitcoin Core's consensus
rule is simply: adopt whichever valid chain has the greatest total work.

A **reorganization** (reorg) is exactly what node A did next: it disconnected
its own top blocks (119–120), reverted their effects (undoing any wallet/UTXO
state changes they caused), and reconnected node B's chain in their place,
walking forward block by block until it reached the new, heavier tip. Any
transactions that existed only in node A's abandoned blocks and nowhere in
node B's chain would return to the mempool (or be dropped if no longer
valid), which is why reorgs are exactly the risk Lab 03's coinbase-maturity
rule and Lab 08's confirmation-depth guidance are designed to protect against.

Nodes choose the greatest-work valid chain — never miner identity, block
arrival time, or any social/off-chain claim — because proof-of-work is the
only property in this system that is objectively measurable, cannot be
forged without redoing the actual computational effort, and cannot be argued
about: any node can independently recompute total chainwork for any chain it
receives and reach the same conclusion, with no need to trust the other
peer's word for who "should" have won.
