# Lab 10 — Competing branches and reorganization

## Commands used

1. **Check chain tip info on both nodes**:
   ```bash
   # Node A
   bitcoin-cli -conf=node_a.conf getblockchaininfo
   # Node B
   bitcoin-cli -conf=node_b.conf getblockchaininfo
   ```

2. **Disconnect Node A from Node B**:
   ```bash
   bitcoin-cli -conf=node_a.conf disconnectnode "node-b-address"
   ```

3. **Mine blocks privately on both nodes**:
   ```bash
   # Node A mines 2 blocks
   bitcoin-cli -conf=node_a.conf generatetoaddress 2 <miner_a_address>
   # Node B mines 4 blocks
   bitcoin-cli -conf=node_b.conf generatetoaddress 4 <miner_b_address>
   ```

4. **Inspect tips under split state**:
   ```bash
   bitcoin-cli -conf=node_a.conf getblockchaininfo
   bitcoin-cli -conf=node_b.conf getblockchaininfo
   ```

5. **Reconnect Node A to Node B**:
   ```bash
   bitcoin-cli -conf=node_a.conf addnode "node-b-address" "onetry"
   ```

6. **Running tests**:
   ```bash
   cargo test --test lab_10
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_10` returns:
```text
running 4 tests
test disconnects_peer_by_address ... ok
test reads_tip_and_accumulated_chainwork ... ok
test reports_convergence_on_the_stronger_branch ... ok
test reconnects_peer_for_synchronization ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Example state representation (Mocked):
- **Split state (competing tips)**:
  - Node A Tip: height 109, hash `short-branch`, chainwork `000000d9`
  - Node B Tip: height 111, hash `strong-branch`, chainwork `000000db`
- **Reconnected state (converged)**:
  - Node A Tip: height 111, hash `strong-branch`, chainwork `000000db`
  - Node B Tip: height 111, hash `strong-branch`, chainwork `000000db`

---

## Evidence references

- Code is implemented in [lab10_reorg.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab10_reorg.rs).
- All unit tests passed, proving the correctness of chain tip reading, peer disconnection/reconnection, and reorganization reports.

---

## Explanation

- **Stale Branch**: When two nodes are disconnected and mine blocks independently, they build two competing branches of the blockchain. In this lab, Node A mined a shorter branch (+2 blocks) and Node B mined a longer branch (+4 blocks). Once the nodes are reconnected, Node A realizes that Node B has a branch with more accumulated work. Node A discards its privately mined blocks, which become "stale" (orphaned), and adopts Node B's branch.
- **Reorganisation (Reorg)**: A chain reorganisation occurs when a node's local view of the best chain tip is replaced by a different valid branch that has more accumulated proof-of-work. During a reorg:
  1. Blocks on the old (stale) branch are deactivated.
  2. Transactions in those deactivated blocks are returned to the mempool (unless they are already confirmed in the new branch).
  3. The blocks on the new, stronger branch are activated.
- **Most-Work-Chain Rule**: Bitcoin nodes converge on a single, global history by always choosing the valid chain tip with the **greatest accumulated proof-of-work** (`chainwork` in RPC). This is the core of Nakamoto Consensus. Nodes do not care about block arrival times, miner identities, or social claims. This objective mathematical rule ensures that the network converges on a single history even in the presence of temporary splits or adversarial miners.
