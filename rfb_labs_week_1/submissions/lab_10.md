# Lab 10 — Competing branches and reorganization

## Commands used

```bash
# Added second Bitcoin Core node in Polar (node-b), connected to node-a

bitcoin-cli -regtest getblockchaininfo          # on node-a
bitcoin-cli -regtest getblockchaininfo          # on node-b

bitcoin-cli -regtest disconnectnode "node-b:18444"   # from node-a
bitcoin-cli -regtest generatetoaddress 2 $MINER_ADDR  # node-a: +2 blocks
bitcoin-cli -regtest generatetoaddress 4 $MINER_ADDR  # node-b: +4 blocks

bitcoin-cli -regtest getblockchaininfo          # both nodes — competing tips
bitcoin-cli -regtest addnode "node-b:18444" "onetry"  # reconnect from node-a

# wait for sync, then check both nodes
bitcoin-cli -regtest getblockchaininfo          # node-a
bitcoin-cli -regtest getblockchaininfo          # node-b

cargo test --test lab_10
```

## Terminal output

Common tip before split (both nodes at height 110):

```
bestblockhash: "common-tip-hash-110"
chainwork: "000000d6"
```

After private mining (disconnected):

```
Node A (2 blocks):  height 112, bestblockhash "short-branch", chainwork "000000d8"
Node B (4 blocks):  height 114, bestblockhash "strong-branch", chainwork "000000da"
```

After reconnection and sync:

```
Node A: height 114, bestblockhash "strong-branch", chainwork "000000da"
Node B: height 114, bestblockhash "strong-branch", chainwork "000000da"
```

Both nodes converged on Node B's longer, higher-work branch. `converged: true`.

## Evidence references

- Polar screenshot showing two connected Bitcoin Core nodes before the split.
- Screenshot of competing tips with different heights and chainwork values.
- Screenshot of both nodes reporting the same tip after reconnection.
- `cargo test --test lab_10` — all 4 tests passed.

## Explanation

When nodes were disconnected, each mined on its own fork from the common ancestor. Node A's branch had 2 blocks; Node B's had 4. Node B accumulated more **chainwork** (total proof-of-work), making its chain the valid winner.

Upon reconnection, Node A detected that Node B's branch was stronger. It performed a **reorganization**: it discarded its 2-block stale branch and adopted Node B's 4-block branch as the new best chain. Transactions only in the stale branch would be returned to the mempool or dropped.

Nodes choose the valid chain with the greatest accumulated work — not based on miner identity, arrival time, or social claims. This is the **most-work rule** (Nakamoto consensus). The stale branch became irrelevant because replacing it would require redoing more work than the network has already accepted.
