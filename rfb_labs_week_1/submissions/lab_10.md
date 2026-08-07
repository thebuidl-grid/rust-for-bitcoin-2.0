# Lab 10 — Competing branches and reorganization

## Commands used

- Node A common tip: `bitcoin-cli getblockchaininfo`
- Node B common tip: `bitcoin-cli -rpcconnect="$POLAR_NODE_B_RPC_HOST" -rpcport="$POLAR_NODE_B_RPC_PORT" getblockchaininfo`
- Peer inspection: `bitcoin-cli getpeerinfo`
- Peer disconnection: `bitcoin-cli disconnectnode "$POLAR_NODE_B_PEER_ADDRESS"`
- Disable Node A networking: `bitcoin-cli setnetworkactive false`
- Disable Node B networking: `bitcoin-cli -rpcconnect="$POLAR_NODE_B_RPC_HOST" -rpcport="$POLAR_NODE_B_RPC_PORT" setnetworkactive false`
- Mine Node A branch: `bitcoin-cli generatetoaddress 2 "$NODE_A_MINER_ADDRESS"`
- Mine Node B branch: `bitcoin-cli -rpcconnect="$POLAR_NODE_B_RPC_HOST" -rpcport="$POLAR_NODE_B_RPC_PORT" generatetoaddress 4 "$NODE_B_MINER_ADDRESS"`
- Restore Node A networking: `bitcoin-cli setnetworkactive true`
- Restore Node B networking: `bitcoin-cli -rpcconnect="$POLAR_NODE_B_RPC_HOST" -rpcport="$POLAR_NODE_B_RPC_PORT" setnetworkactive true`
- Reconnection: `bitcoin-cli addnode "$POLAR_NODE_B_PEER_ADDRESS" onetry`
- Final Node A tip: `bitcoin-cli getblockchaininfo`
- Final Node B tip: `bitcoin-cli -rpcconnect="$POLAR_NODE_B_RPC_HOST" -rpcport="$POLAR_NODE_B_RPC_PORT" getblockchaininfo`

## Terminal output

```
{
  "common_tip_before_split": {
    "best_block_hash": "65cee207efa3c03e9a7cc72dc2cb080d2df616b3b3f5ec350bebfa6b39dd99f7",
    "chainwork": "00000000000000000000000000000000000000000000000000000000000000de",
    "height": 110
  },
  "report": {
    "common_tip_before_split": "65cee207efa3c03e9a7cc72dc2cb080d2df616b3b3f5ec350bebfa6b39dd99f7",
    "competing_tips": {
      "node_a": {
        "best_block_hash": "62449e6108af145b0d4615da4de0f435f6c3394326f097e00486e1459ba928cd",
        "chainwork": "00000000000000000000000000000000000000000000000000000000000000e2",
        "height": 112
      },
      "node_b": {
        "best_block_hash": "46c7079be53b6f65d587020dd969a91c515fea521ec226e4d396699d555f7c52",
        "chainwork": "00000000000000000000000000000000000000000000000000000000000000e6",
        "height": 114
      }
    },
    "converged": true,
    "final_tips": {
      "node_a": {
        "best_block_hash": "46c7079be53b6f65d587020dd969a91c515fea521ec226e4d396699d555f7c52",
        "chainwork": "00000000000000000000000000000000000000000000000000000000000000e6",
        "height": 114
      },
      "node_b": {
        "best_block_hash": "46c7079be53b6f65d587020dd969a91c515fea521ec226e4d396699d555f7c52",
        "chainwork": "00000000000000000000000000000000000000000000000000000000000000e6",
        "height": 114
      }
    }
  }
}
```

## Evidence references
![alt text](evidence/image-9.png)

## Explanation

While disconnected, both nodes can extend the shared history independently. After
reconnection, the valid four-block branch has more accumulated proof of work than
the valid two-block branch. Node A therefore disconnects its two private blocks and
activates Node B's branch; this change of active history is a reorganization, and
Node A's displaced branch becomes stale. Selection is based on greatest accumulated
work, not miner identity, which branch arrived first, or a social claim about which
history should win.
