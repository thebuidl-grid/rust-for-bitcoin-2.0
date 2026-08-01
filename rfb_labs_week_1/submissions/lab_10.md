# Lab 10 — Competing branches and reorganization

## Commands used

```
cargo test --test lab_10

# common tip, both nodes synced
bitcoin-cli -regtest -rpcconnect=<node-a> getblockchaininfo
bitcoin-cli -regtest -rpcconnect=<node-b> getblockchaininfo

# split the network
bitcoin-cli -regtest -rpcconnect=<node-a> disconnectnode "<node-b-address>"

# mine a shorter branch on node A, a longer branch on node B
bitcoin-cli -regtest -rpcconnect=<node-a> generatetoaddress <n> "<node-a-address>"
bitcoin-cli -regtest -rpcconnect=<node-b> generatetoaddress <m> "<node-b-address>"

# competing tips
bitcoin-cli -regtest -rpcconnect=<node-a> getblockchaininfo
bitcoin-cli -regtest -rpcconnect=<node-b> getblockchaininfo

# reconnect and let the most-work chain win
bitcoin-cli -regtest -rpcconnect=<node-a> addnode "<node-b-address>" onetry

# final, converged tips
bitcoin-cli -regtest -rpcconnect=<node-a> getblockchaininfo
bitcoin-cli -regtest -rpcconnect=<node-b> getblockchaininfo
```

*RPCs are the ones issued by `get_chain_tip`, `disconnect_peer`, and `reconnect_peer` in `src/labs/lab10_reorg.rs`, verified against the mocked RPC client in `tests/lab_10.rs`; `build_reorg_report` compares the recorded snapshots locally (no RPC). Run the `bitcoin-cli` lines against two live Polar regtest nodes, mining more blocks on node B than node A (`m > n`) so its branch wins, to capture the terminal output below.*

## Terminal output

Captured against two real, separate regtest nodes (`bitcoind-lab-a`, `bitcoind-lab-b`, Bitcoin Core 30.0), continuing from node A's state at the end of Lab 09 (height 108); node B started fresh and was connected first so it could IBD-sync to a common tip:

```
$ bitcoin-cli -regtest -rpcconnect=node-a addnode "bitcoind-lab-b:18444" onetry

# common tip before split
$ bitcoin-cli -regtest -rpcconnect=node-a getblockchaininfo
{ "blocks": 108, "bestblockhash": "57215f15b07ecff63091c8e863960e04f0a6002b90f0323011e72b65247bc104", "chainwork": "...da", ... }
$ bitcoin-cli -regtest -rpcconnect=node-b getblockchaininfo
{ "blocks": 108, "bestblockhash": "57215f15b07ecff63091c8e863960e04f0a6002b90f0323011e72b65247bc104", "chainwork": "...da", ... }

# split the network
$ bitcoin-cli -regtest -rpcconnect=node-a disconnectnode "bitcoind-lab-b:18444"

# short branch on A (2 blocks), longer branch on B (4 blocks)
$ bitcoin-cli -regtest -rpcconnect=node-a generatetoaddress 2 "<node-a-address>"
[ "11203da91f6a5b2669564174996a5038c5d4f002a5d472e07cc512be4da27709", "0c1c671699c50293f036093503a96f403e637f1a500bb4448e5f0d76517b7818" ]
$ bitcoin-cli -regtest -rpcconnect=node-b generatetoaddress 4 "<node-b-address>"
[ "183dd51d...", "36944aa9...", "5f29e002...", "21fabb13d63e293d2c0e454bae69f711c0ba046ac43b3ca55e4f1808842995c1" ]

# competing tips
$ bitcoin-cli -regtest -rpcconnect=node-a getblockchaininfo
{ "blocks": 110, "bestblockhash": "0c1c671699c50293f036093503a96f403e637f1a500bb4448e5f0d76517b7818", "chainwork": "...de", ... }
$ bitcoin-cli -regtest -rpcconnect=node-b getblockchaininfo
{ "blocks": 112, "bestblockhash": "21fabb13d63e293d2c0e454bae69f711c0ba046ac43b3ca55e4f1808842995c1", "chainwork": "...e2", ... }

# reconnect
$ bitcoin-cli -regtest -rpcconnect=node-a addnode "bitcoind-lab-b:18444" onetry

# final tips — converged
$ bitcoin-cli -regtest -rpcconnect=node-a getblockchaininfo
{ "blocks": 112, "bestblockhash": "21fabb13d63e293d2c0e454bae69f711c0ba046ac43b3ca55e4f1808842995c1", "chainwork": "...e2", ... }
$ bitcoin-cli -regtest -rpcconnect=node-b getblockchaininfo
{ "blocks": 112, "bestblockhash": "21fabb13d63e293d2c0e454bae69f711c0ba046ac43b3ca55e4f1808842995c1", "chainwork": "...e2", ... }
```

Before the split, both nodes shared height 108 / hash `57215f15...`. After splitting and mining independently, A reached height 110 (chainwork `...de`) while B reached height 112 with strictly greater chainwork (`...e2`). After reconnecting, **A abandoned its own 2-block branch** (`11203da9...`, `0c1c6716...` are no longer A's tip) **and adopted B's 4-block branch** — both nodes converged on height 112, hash `21fabb13...`, matching chainwork `...e2`. This is a real, observed reorg: A's blocks became stale/orphaned the moment a chain with more accumulated proof of work arrived.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a/-b bitcoin-cli ...` against two real regtest nodes on an isolated Docker network (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

**Most-work-chain rule:** Bitcoin nodes don't follow the *longest* chain by block count — they follow the chain with the most cumulative **proof of work**, tracked by `chainwork`. Almost always that's also the chain with more blocks, but it's work, not length, that actually decides. Here, B's 4 blocks (chainwork `...e2`) outweighed A's 2 blocks (chainwork `...de`), so B's chain won even though nothing about A's blocks was individually invalid.

**Reorganization:** while disconnected, A and B each kept extending the chain they could see — a normal, unavoidable situation any time the network partitions or two miners find blocks near-simultaneously. Once reconnected, each node compared the chainwork of its own tip against the chain the other node offered. A saw that B's chain had more accumulated work than its own, so it **reorganized**: it rolled back its own two blocks and adopted B's four blocks instead, ending up at the exact same tip (`21fabb13...`, height 112) as B.

**Stale branch:** A's two abandoned blocks (`11203da9...`, `0c1c6716...`) are now a **stale branch** — valid blocks that were mined correctly, but that are no longer part of the chain everyone agrees on. Any transactions that existed *only* in those blocks (and nowhere in B's chain) go back to being unconfirmed and return to the mempool to potentially be mined again later; any coinbase rewards from those two blocks simply vanish, since they were never part of the winning chain. This is precisely the mechanism the 100-block coinbase-maturity rule in Lab 03 exists to protect against — deep reorgs are rare, but shallow ones like this happen naturally, and the maturity rule ensures a reorg can never make a wallet think it has already-spent money that a rollback just erased.
