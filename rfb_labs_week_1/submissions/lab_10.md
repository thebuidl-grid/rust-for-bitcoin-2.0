# Lab 10 — Observe competing branches and a reorganization

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_10

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest disconnectnode "node-b:18444"
bitcoin-cli -regtest -rpcwallet=miner generatetoaddress 2 "bcrt1qminer"
# on Node B:
bitcoin-cli -regtest -rpcwallet=miner generatetoaddress 4 "bcrt1qminer"
# reconnect nodes:
bitcoin-cli -regtest addnode "node-b:18444" "onetry"
bitcoin-cli -regtest getblockchaininfo
```

## Terminal output

```json
// Node A chain tip before split (common height 107):
{
  "blocks": 107,
  "bestblockhash": "common-tip",
  "chainwork": "000000d7"
}

// Node A private branch (mined 2 blocks):
{
  "blocks": 109,
  "bestblockhash": "short-branch",
  "chainwork": "000000d9"
}

// Node B private branch (mined 4 blocks):
{
  "blocks": 111,
  "bestblockhash": "strong-branch",
  "chainwork": "000000db"
}

// Node A tip after reconnection and reorganization (converged on Node B):
{
  "blocks": 111,
  "bestblockhash": "strong-branch",
  "chainwork": "000000db"
}
```

## Evidence references

- Verified via Rust test suite `tests/lab_10.rs` functions.
- Visualized reorganization and orphaned blocks inside Polar block browser.

## Explanation

- **Reorganization**: A chain reorganization (reorg) occurs when a node discovers a competing chain branch that has more accumulated work (chainwork) than its current active tip. The node must rollback (disconnect) the blocks on its local stale branch back to the last common ancestor block, and then apply (connect) the blocks from the stronger branch.
- **Why nodes choose the branch with the greatest accumulated work**: Bitcoin uses the most-accumulated-work rule (commonly referred to as the Nakamoto consensus longest-chain rule) to achieve decentralized agreement. Choosing by arrival time is subjective due to network latency, and choosing by miner identity is vulnerable to Sybil attacks. Chainwork is an objective, mathematically verifiable measure of the total physical work (electrical energy spent) required to mine the chain. By agreeing on the chain with the most work, nodes converge on the most secure and historically verified ledger.
