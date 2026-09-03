# Lab 10 — Competing branches and reorganization

## Commands used

Used `disconnectnode`, `setnetworkactive false`, `generatetoaddress` on each isolated node, then `setnetworkactive true`, `addnode backend2 onetry`, and `getblockchaininfo`.

## Terminal output

Common tip: height 108, `4f5e61d81f0b8ba6a7cfc033a4f26fb2aeeb7522cb717956079d69521ef97a9e`, chainwork `00da`. Node A's 2-block branch reached height 110 and chainwork `00de`; Node B's 4-block branch reached height 112, tip `08d7aff130a02b1aae515297ef90e7be40d4dd86161c11ae4fb7b389e5005d10`, chainwork `00e2`. After reconnection both nodes reported that same height-112 tip.

## Evidence references

Terminal snapshots from `polar-n1-backend1` and `polar-n1-backend2` record common, private, and final tips. Final Node A peer data reported synced headers and blocks at height 112.

## Explanation

Node A's shorter valid branch became stale when it learned Node B's branch with greater accumulated work. A reorganization changes the active tip to that valid most-work branch; selection is not based on miner identity, social claims, or arrival time.
