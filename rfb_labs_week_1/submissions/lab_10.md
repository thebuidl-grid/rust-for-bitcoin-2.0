# Lab 10 — Competing branches and reorganization

## Commands used

cargo run --example lab10 -- 18444 172.21.0.3:42374 172.21.0.3:18444 bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn bcrt1qsn66g9yafu77yk5uaexeldhjfegv6psd4fy2ya

Underlying bitcoin-cli RPCs invoked:
- getblockchaininfo                                    (node A, node B — before split)
- disconnectnode 172.21.0.3:42374                      (node A)
- generatetoaddress 2 bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn   (node A)
- generatetoaddress 4 bcrt1qsn66g9yafu77yk5uaexeldhjfegv6psd4fy2ya            (node B)
- getblockchaininfo                                    (node A, node B — private tips)
- addnode 172.21.0.3:18444 onetry                       (node A)
- getblockchaininfo                                    (node A, node B — polled until matching)


## Terminal output

common tip before split: height=111 hash=6ce9a3677f5b5262f2642c8ae920653a470dcf6d48eef2aeff636aa50e289866 chainwork=00000000000000000000000000000000000000000000000000000000000000e0
node A private tip: height=113 hash=5a211d38731262e44ef5bf7a50fc73c8c3bb5bd10d75dbc8e3b58e5f4e8b7b39 chainwork=00000000000000000000000000000000000000000000000000000000000000e4
node B private tip: height=115 hash=7115e7db9d07df20c0a01fa9261f33783900ebd562ac390cf30a4821becbf4d7 chainwork=00000000000000000000000000000000000000000000000000000000000000e8
converged: true
final node A tip: height=115 hash=7115e7db9d07df20c0a01fa9261f33783900ebd562ac390cf30a4821becbf4d7
final node B tip: height=115 hash=7115e7db9d07df20c0a01fa9261f33783900ebd562ac390cf30a4821becbf4d7

## Evidence references

- getblockchaininfo                                    (node A, node B — before split)
- disconnectnode 172.21.0.3:42374                      (node A)
- generatetoaddress 2 bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn   (node A)
- generatetoaddress 4 bcrt1qsn66g9yafu77yk5uaexeldhjfegv6psd4fy2ya            (node B)
- getblockchaininfo                                    (node A, node B — private tips)
- addnode 172.21.0.3:18444 onetry                       (node A)
- getblockchaininfo                                    (node A, node B — polled until matching)

## Explanation

A stale branch happens when two miners find a valid block at roughly the same height around the same time, for a brief window, the network has two competing valid chains extending from the same parent, and different nodes may initially see different "tips" depending on which block reached them first.
A reorganization is what a node does when it learns about a competing chain that is now longer, or has more accumulated work, than the one it currently considers best.
The most-work-chain rule is the deterministic tiebreaker that makes the resolution possible and eventually consistent: every node is programmed to always consider the chain with the greatest cumulative proof-of-work as the canonical one, not the longest chain by block count and not whichever block saw it first