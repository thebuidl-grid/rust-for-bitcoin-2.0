# Lab 10 — Competing branches and reorganization

## Commands used

```bash
cargo test --test lab_10
```

RPC methods called:
- `getblockchaininfo` - Get chain tip height, hash, and accumulated chainwork
- `disconnectnode <peer>` - Disconnect from peer to create isolated branches
- `addnode <peer> onetry` - Reconnect peer for one-time synchronization attempt

## Terminal output

```
running 4 tests
test disconnects_peer_by_address ... ok
test reads_tip_and_accumulated_chainwork ... ok
test reconnects_peer_for_synchronization ... ok
test reports_convergence_on_the_stronger_branch ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, demonstrating:
- Reading chain tip with accumulated chainwork
- Disconnecting peers to create isolated branches
- Reconnecting triggers synchronization
- Nodes converge when both reach the same height and block hash

## Explanation

Lab 10 demonstrates Bitcoin's consensus mechanism through reorganization and the most-work rule:

1. **Competing Branches**: Network partitions or mining races create competing blockchain branches. If isolated, nodes build on different tips, creating blockchain forks.

2. **The Most-Work Rule**: Bitcoin consensus selects the chain with accumulated most work (highest chainwork):
   - Chainwork represents cumulative difficulty from genesis
   - Each block adds `2^256 / difficulty_target` to chainwork
   - This incentivizes following the branch with most miner investment

3. **Reorganization (Reorg)**: When nodes reconnect:
   - They compare chainwork of their tips
   - The losing branch's blocks become "orphaned"
   - Nodes switch to the higher-work branch
   - This is a "reorganization" - past transactions are reorganized or reversed

4. **Convergence**: If both branches have equal chainwork, nodes may temporarily diverge, but honest nodes eventually align on the longest chain (Bitcoin's Nakamoto consensus).

5. **Security Model**: Bitcoin's security relies on:
   - Proof-of-work making it expensive to create alternate branches
   - Most-work consensus encouraging honest miners to follow the main chain
   - Confirmation depth making reorganizations exponentially harder

6. **Practical Implications**:
   - Transactions can be reversed with chain reorganization
   - 6+ confirmations provide security against 51% attacks
   - This is why Bitcoin requires time (confirmations) for settlement
   - Competing branches are rare with honest miners controlling >50% of hashpower
