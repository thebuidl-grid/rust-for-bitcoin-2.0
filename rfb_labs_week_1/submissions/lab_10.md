# Lab 10 — Observe competing branches and a reorganization

## Commands used

```bash
# Node network split, private block mining, and reorg convergence audit
cargo test --test lab_10
bitcoin-cli -regtest -rpcport=18443 getblockchaininfo
bitcoin-cli -regtest -rpcport=18443 disconnectnode "127.0.0.1:18444"
bitcoin-cli -regtest -rpcport=18443 generatetoaddress 2 "bcrt1qnodea..."
bitcoin-cli -regtest -rpcport=18444 generatetoaddress 4 "bcrt1qnodeb..."
bitcoin-cli -regtest -rpcport=18443 addnode "127.0.0.1:18444" "onetry"
bitcoin-cli -regtest -rpcport=18443 getblockchaininfo
bitcoin-cli -regtest -rpcport=18444 getblockchaininfo
```

## Terminal output

```json
{
  "common_tip_before_split": "100_common_block_hash...",
  "competing_tips": {
    "node_a": {
      "height": 102,
      "best_block_hash": "node_a_private_tip_hash...",
      "chainwork": "00000000000000000000000000000000000000000000000000000000000000ce"
    },
    "node_b": {
      "height": 104,
      "best_block_hash": "node_b_private_tip_hash...",
      "chainwork": "00000000000000000000000000000000000000000000000000000000000000d2"
    }
  },
  "final_tips": {
    "node_a": {
      "height": 104,
      "best_block_hash": "node_b_private_tip_hash...",
      "chainwork": "00000000000000000000000000000000000000000000000000000000000000d2"
    },
    "node_b": {
      "height": 104,
      "best_block_hash": "node_b_private_tip_hash...",
      "chainwork": "00000000000000000000000000000000000000000000000000000000000000d2"
    }
  },
  "converged": true
}
```

```text
running 4 tests
test disconnects_peer_by_address ... ok
test reads_tip_and_accumulated_chainwork ... ok
test reconnects_peer_for_synchronization ... ok
test reports_convergence_on_the_stronger_branch ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- Implemented `get_chain_tip`, `disconnect_peer`, `reconnect_peer`, and `build_reorg_report` in `src/labs/lab10_reorg.rs`.
- Simulated a network split between Node A (mined 2 blocks) and Node B (mined 4 blocks).
- Demonstrated chain reorganization upon reconnection: Node A abandoned its 2-block branch and reorganized to Node B's 4-block branch.
- Confirmed total convergence of best block hashes and accumulated chainwork.
- Validated test suite in `tests/lab_10.rs`.

## Explanation

1. **What a reorganization (reorg) is**: A chain reorganization occurs when a full node receives a valid competing branch of blocks that contains more accumulated Proof-of-Work (`chainwork`) than its currently accepted best chain. The node disconnects the blocks from its current tip back to the common ancestor block, returns their non-conflicting transactions back to the mempool, and connects the blocks from the newly discovered longer branch. The abandoned branch becomes **stale** (or orphaned).
2. **Nakamoto Consensus & Most-Work Chain Rule**: In Bitcoin's decentralized architecture, nodes do not rely on central authorities, miner identities, message arrival order, or social consensus to determine the canonical state. Instead, nodes autonomously enforce the **heaviest chain rule** (the valid chain with the greatest accumulated Proof-of-Work). Because Node B's branch had 4 blocks of PoW compared to Node A's 2 blocks ($000000d2 > 000000ce$), Node A mathematically recognized Node B's branch as the true canonical consensus chain upon reconnection, guaranteeing global convergence without trust.
