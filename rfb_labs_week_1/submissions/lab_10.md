# Lab 10 — Competing branches and reorganization

## Commands used

```bash
# Read the common tip and peer addresses on both nodes.
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getblockchaininfo
docker exec --user bitcoin polar-n2-backend2 bitcoin-cli -regtest getblockchaininfo
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getpeerinfo
docker exec --user bitcoin polar-n2-backend2 bitcoin-cli -regtest getpeerinfo

# Disconnect the named peers, then keep automatic Polar peers offline during mining.
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest disconnectnode backend2
docker exec --user bitcoin polar-n2-backend2 bitcoin-cli -regtest disconnectnode backend1
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest setnetworkactive false
docker exec --user bitcoin polar-n2-backend2 bitcoin-cli -regtest setnetworkactive false

# Mine unequal private branches from the same height-110 tip.
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner \
  generatetoaddress 2 bcrt1q2gung50cdgh8ka6ptg4d9t7z2kv8fj09z5rqfh
docker exec --user bitcoin polar-n2-backend2 bitcoin-cli -regtest \
  generatetoaddress 4 bcrt1qsjzyxsrumtyj97yvpwd3p8k3qqjyucsgr9c0pw

# Re-enable networking and request one-time synchronization.
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest setnetworkactive true
docker exec --user bitcoin polar-n2-backend2 bitcoin-cli -regtest setnetworkactive true
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest addnode backend2 onetry

# Inspect convergence and the stale short branch.
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getblockchaininfo
docker exec --user bitcoin polar-n2-backend2 bitcoin-cli -regtest getblockchaininfo
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getchaintips
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest \
  getblockheader 10896d299c51a06267bf408557f863ee4b0614428a80a3f667c03e188f8d1ae1
```

## Terminal output

```text
Common tip before split (both nodes):
  height:    110
  hash:      1f3f9ee7bd0af8bf1910b47b7bf7abaf5bcface9c79a7eb446983fede457d9f6
  chainwork: 00000000000000000000000000000000000000000000000000000000000000de

After disconnectnode + setnetworkactive false:
  backend1 connections: 0
  backend2 connections: 0

backend1 private branch (2 blocks):
  blocks:    112
  tip:       10896d299c51a06267bf408557f863ee4b0614428a80a3f667c03e188f8d1ae1
  chainwork: 00000000000000000000000000000000000000000000000000000000000000e2

backend2 private branch (4 blocks):
  blocks:    114
  tip:       7f286941b14e1ef11f96f49cd0d3f29a3fa2a3b1baa51d5a6c4804c91300347a
  chainwork: 00000000000000000000000000000000000000000000000000000000000000e6

Final tip after reconnection (both nodes):
  blocks:    114
  tip:       7f286941b14e1ef11f96f49cd0d3f29a3fa2a3b1baa51d5a6c4804c91300347a
  chainwork: 00000000000000000000000000000000000000000000000000000000000000e6

getchaintips on backend1:
  height 114, hash 7f286941...00347a, branchlen 0, status active
  height 112, hash 10896d29...8d1ae1, branchlen 2, status valid-fork

Former backend1 tip header:
  height: 112
  confirmations: -1
  chainwork: 00000000000000000000000000000000000000000000000000000000000000e2
```

## Evidence references

Live Polar v4.0.0 network `Week 1 Bitcoin Fundamentals`, containing connected Bitcoin
Core v30.0 nodes `backend1` and `backend2`. Evidence consists of the Polar network view
and the live `getblockchaininfo`, `getpeerinfo`, mining, `getchaintips`, and stale-block
header transcripts recorded above. All commands ran only on the local regtest network.

## Explanation

Both nodes began from the same height-110 block. While partitioned, backend1 added two
blocks and backend2 added four, so each node temporarily considered its own private tip
active. On reconnection, backend1 learned about backend2's branch and reorganized to it
because its chainwork (`...00e6`) exceeded the short branch's chainwork (`...00e2`). The
two tips then converged on the same height-114 hash. Bitcoin Core retained the displaced
two-block branch as a `valid-fork`; its old tip has `-1` confirmations because it is valid
but no longer part of the active most-work chain. This demonstrates that nodes select the
valid chain with the greatest accumulated proof of work, not merely the chain they saw
first.
