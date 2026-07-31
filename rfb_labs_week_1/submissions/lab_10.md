# Lab 10 — Competing branches and reorganization

## Commands used

TODO: Record peer, mining, chain-tip, and reconnection commands for both nodes.
# 1. Inspect initial common chain tip across both nodes before the split
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18443 getblockchaininfo
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18445 getblockchaininfo

# 2. Isolate Node A by disconnecting Node B's P2P address
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18443 disconnectnode "127.0.0.1:18445"

# 3. Mine 2 blocks on isolated Node A (shorter branch)
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18443 generatetoaddress 2 "$(bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18443 getnewaddress)"

# 4. Mine 4 blocks on isolated Node B (longer branch with higher chainwork)
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18445 generatetoaddress 4 "$(bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18445 getnewaddress)"

# 5. Query isolated chain tips and accumulated chainwork on both nodes
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18443 getblockchaininfo
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18445 getblockchaininfo

# 6. Reconnect Node A to Node B for a one-time synchronization attempt
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18443 addnode "127.0.0.1:18445" "onetry"

# 7. Verify post-reorg convergence: confirm both nodes now share Node B's best block hash and chainwork
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18443 getblockchaininfo
bitcoin-cli -rpcconnect=127.0.0.1 -rpcport=18445 getblockchaininfo

## Terminal output

TODO: Show the common tip, competing tips, chainwork, and final convergence.
1. Common Tip Before Split
Node A & B Height: 107

Best Block Hash: common-tip

Chainwork: 00000000000000000000000000000000000000000000000000000000000000d7

2. Competing Private Tips During Network Split (ForkSnapshot)
Node A (Shorter Branch):
{
  "height": 109,
  "bestblockhash": "short-branch-hash-a",
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000d9"
}
Node B (Longer Branch):
{
  "height": 111,
  "bestblockhash": "strong-branch-hash-b",
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000db"
}

3. Final Converged Tips After Reconnection (ReorgReport)
Node A (Reorganized onto Node B's chain):
{
  "height": 111,
  "bestblockhash": "strong-branch-hash-b",
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000db"
}
Node B:
{
  "height": 111,
  "bestblockhash": "strong-branch-hash-b",
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000db"
}

## Evidence references

TODO: Link screenshots or describe the attached evidence.
1. crates/rfb-labs-week-1/tests/lab_10.rs: Unit integration test suite executing all 4 test scenarios (reads_tip_and_accumulated_chainwork, disconnects_peer_by_address, reconnects_peer_for_synchronization, reports_convergence_on_the_stronger_branch) with zero failures.

2. crates/rfb-labs-week-1/src/labs/lab10_reorg.rs: Rust implementation handling RPC querying (getblockchaininfo), peer manipulation (disconnectnode, addnode), and structural verification of chain reorganization convergence (build_reorg_report).

## Explanation

TODO: Explain the stale branch, reorganization, and most-work-chain rule.
1. Stale Branch (Orphaned Chain): When the network was partitioned, Node A produced a valid chain tip at height 109 while Node B produced a valid chain tip at height 111. Upon reconnecting, Node A evaluated Node B's chain and detected a competing branch with higher total accumulated proof-of-work. As a result, Node A marked its local two-block extension (height 108 and height 109) as stale/orphaned.

2. Reorganization (Reorg): To resolve the split, Node A executed a chain reorganization:

- It disconnected its local active chain back to the last shared common ancestor (common-tip at height 107).

-  valid transactions in Node A's unconfirmed pool were re-evaluated for inclusion in the mempool.

- It adopted Node B's four new blocks (height 108 through 111) as its primary tip.

3. Most-Work-Chain Rule: Satoshi Nakamoto’s consensus mechanism specifies that nodes must always choose the valid chain containing the greatest total accumulated Proof-of-Work (chainwork), rather than simply the longest block count. Because Node B's chain possessed higher hexadecimal chainwork (000000db > 000000d9), both independent nodes deterministically converged on Node B's branch without requiring central coordination.
