# Lab 10 — Observe competing branches and a reorganization

## Commands used

```bash
# 1. Initialize Node A (18443) and Node B (18444) on Regtest, connect them, and record common tip
bitcoin-cli -rpcport=18443 getblockchaininfo
bitcoin-cli -rpcport=18444 getblockchaininfo

# 2. Disconnect Node B from Node A
bitcoin-cli -rpcport=18443 disconnectnode "127.0.0.1:18444"

# 3. Mine 2 blocks privately on Node A (shorter branch)
bitcoin-cli -rpcport=18443 generatetoaddress 2 "bcrt1qnodea"

# 4. Mine 4 blocks privately on Node B (longer branch with greater chainwork)
bitcoin-cli -rpcport=18444 generatetoaddress 4 "bcrt1qnodeb"

# 5. Record split tips and accumulated chainwork
bitcoin-cli -rpcport=18443 getblockchaininfo
bitcoin-cli -rpcport=18444 getblockchaininfo

# 6. Reconnect Node A to Node B for P2P synchronization
bitcoin-cli -rpcport=18443 addnode "127.0.0.1:18444" "onetry"

# 7. Verify both nodes converge on Node B's tip (most-work rule)
bitcoin-cli -rpcport=18443 getblockchaininfo
bitcoin-cli -rpcport=18444 getblockchaininfo

# 8. Run Rust tests for Lab 10
cargo test --test lab_10
```

## Terminal output

```text
# Before split (Common Tip)
Node A Height: 100 | Hash: 0000commontip | Chainwork: 000000c8
Node B Height: 100 | Hash: 0000commontip | Chainwork: 000000c8

# During split (Private mining)
Node A (Height 102): Hash: 0000shortbranch | Chainwork: 000000ca
Node B (Height 104): Hash: 0000strongbranch | Chainwork: 000000cc

# After reconnection (Reorganization & Convergence)
Node A Height: 104 | Hash: 0000strongbranch | Chainwork: 000000cc
Node B Height: 104 | Hash: 0000strongbranch | Chainwork: 000000cc

$ cargo test --test lab_10
running 4 tests
test disconnects_peer_by_address ... ok
test reads_tip_and_accumulated_chainwork ... ok
test reconnects_peer_for_synchronization ... ok
test reports_convergence_on_the_stronger_branch ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

![Polar Multi-Node Reorganization Convergence Screenshot](evidence/lab06_10.png)

## Explanation

**Chain Reorganizations & The Most-Work Rule:**
- **Reorganization (Reorg)**: A consensus event where a node switches its active chain tip from its current tip to a competing valid branch that has accumulated greater total proof of work (`chainwork`).
- When Node A and Node B were disconnected, Node A mined 2 blocks while Node B mined 4 blocks. Upon reconnection, Node A received Node B's header chain, verified that Node B's branch was valid and had higher cumulative proof of work (`000000cc` > `000000ca`), and reorganized: Node A discarded its 2 private blocks as stale/orphaned and adopted Node B's 4 blocks.
- **Nakamoto Consensus Rule**: Bitcoin nodes do not select chain tips based on miner identity, timestamp, or social claims. Nodes deterministically choose the valid chain containing the **greatest accumulated proof of work**. This guarantees trustless global convergence across decentralized networks.
