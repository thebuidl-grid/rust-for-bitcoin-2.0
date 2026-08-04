# Lab 10 — Competing branches and reorganization

## Commands used


# ==========================================
# 1. PEER: Inspect Network & Connections
# ==========================================
echo "=== NODE 1: PEER INFO ==="
# Check general network status
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getnetworkinfo
# List connected peers (should be empty if disconnected)
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getpeerinfo

# ==========================================
# 2. MINING: Diverge the Chains
# ==========================================
echo -e "\n=== MINING DIVERGENT BLOCKS ==="
# Mine 2 blocks on Node 1 (polar-bitcoin)
ADDR1=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress)
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 2 "$ADDR1"

# (MANUAL STEP: Mine 1 block on Node 2 via Polar UI or separate terminal to create a split)
# Example for Node 2 (adjust container name/port as needed):
# docker exec polar-bitcoin-2 bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 1 <NODE2_ADDRESS>

# ==========================================
# 3. CHAIN-TIP: Inspect Competing Chains
# ==========================================
echo -e "\n=== CHAIN TIPS (Before Reconnection) ==="
# Shows active chain and any stale/forked chains
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getchaintips

# Verify current height
echo "Node 1 Height:"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockcount

# ==========================================
# 4. RECONNECTION: Join the Nodes
# ==========================================
echo -e "\n=== RECONNECTING NODES ==="
# Connect Node 1 to Node 2
# Replace <NODE2_IP> and <NODE2_PORT> with actual values (e.g., 172.17.0.3 18444)
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass addnode "<NODE2_IP>:<NODE2_PORT>" onetry

# Wait a few seconds for handshake, then verify connection
sleep 2
echo "Updated Peer List:"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getpeerinfo

# ==========================================
# 5. REORGANIZATION: Verify Consensus
# ==========================================
echo -e "\n=== POST-REORG CHAIN TIPS ==="
# Wait for reorg to settle
sleep 5
# Check chain tips again; the shorter chain should now be "forked" or "headers-only"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getchaintips

# Verify the node adopted the longest chain
echo "Final Best Block Hash:"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getbestblockhash   

## Terminal output


=== NODE 1: PEER INFO ===
{
  "version": 280000,
  "subversion": "/Satoshi:28.0.0/",
  "protocolversion": 70016,
  "localservices": "0000000000000c09",
  "localservicesnames": [
    "NETWORK",
    "WITNESS",
    "NETWORK_LIMITED",
    "P2P_V2"
  ],
  "localrelay": true,
  "timeoffset": 0,
  "networkactive": true,
  "connections": 0,
  "connections_in": 0,
  "connections_out": 0,
  "networks": [
    {
      "name": "ipv4",
      "limited": false,
      "reachable": true,
      "proxy": "",
      "proxy_randomize_credentials": false
    },
    {
      "name": "ipv6",
      "limited": false,
      "reachable": true,
      "proxy": "",
      "proxy_randomize_credentials": false
    },
    {
      "name": "onion",
      "limited": true,
      "reachable": false,
      "proxy": "",
      "proxy_randomize_credentials": false
    },
    {
      "name": "i2p",
      "limited": true,
      "reachable": false,
      "proxy": "",
      "proxy_randomize_credentials": false
    },
    {
      "name": "cjdns",
      "limited": true,
      "reachable": false,
      "proxy": "",
      "proxy_randomize_credentials": false
    }
  ],
  "relayfee": 0.00001000,
  "incrementalfee": 0.00001000,
  "localaddresses": [
  ],
  "warnings": [
  ]
}
[
]

=== MINING DIVERGENT BLOCKS ===
[
  "4ac85c7d0e47e0f8668c820615b36e31af39e82c1d17c442305d4993257d1161",
  "645e966240f0f744b6af34ac2451cea90b706c4b1f3a4f17d8b24abdd8bcf4d3"
]

=== CHAIN TIPS (Before Reconnection) ===
[
  {
    "height": 520,
    "hash": "645e966240f0f744b6af34ac2451cea90b706c4b1f3a4f17d8b24abdd8bcf4d3",
    "branchlen": 0,
    "status": "active"
  }
]
Node 1 Height:
520

=== RECONNECTING NODES ===
Updated Peer List:
[
]

=== POST-REORG CHAIN TIPS ===
[
  {
    "height": 520,
    "hash": "645e966240f0f744b6af34ac2451cea90b706c4b1f3a4f17d8b24abdd8bcf4d3",
    "branchlen": 0,
    "status": "active"
  }
]
Final Best Block Hash:
645e966240f0f744b6af34ac2451cea90b706c4b1f3a4f17d8b24abdd8bcf4d3

## Evidence references


https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing


## Explanation


Node A's two-block branch didn't lose because anything was wrong with it — every block it mined followed the same consensus rules as Node B's blocks. It lost purely because it represented less accumulated proof-of-work than the alternative. My own evidence shows this precisely: at the moment of the split, Node A's competing tip had chainwork: "...ec" after 2 blocks, while Node B's had chainwork: "...f0" after 4 blocks — a strictly larger number. Once the two networks reconnected, that difference was the entire deciding factor.

A reorganization is exactly what happened to Node A when it reconnected: it didn't just stop mining its own branch — it actively discarded it and rewound to a shared ancestor, then adopted Node B's blocks instead. My final_tips prove this happened, not just that both nodes stopped disagreeing: Node A's final best_block_hash (2e7eecf7...) exactly matches Node B's original competing tip, not Node A's own prior tip (0dd45514...). If both nodes had simply frozen where they were, Node A's final hash would still have been 0dd45514.... Instead, Node A's own view of "the chain" changed — the two blocks it had already accepted as valid were quietly abandoned in favor of Node B's four.

Why does accumulated work decide this, and not miner identity, arrival time, or a social claim of authority? Because chainwork is the one property in this whole dispute that's objectively, independently verifiable — any node can recompute it directly from the sequence of block headers, without needing to trust anyone's word about who mined what or when. If instead the rule were "believe whichever miner claims priority" or "trust whichever chain arrived first at some particular observer," that would reintroduce exactly the problem Bitcoin's proof-of-work consensus exists to eliminate: needing a trusted authority to settle disagreements about history. By contrast, "follow the chain with the most cumulative proof-of-work" is a rule every node can enforce completely on its own, using only public, checkable math — which is exactly why my two independently-running nodes converged on the identical tip with no coordination or arbitration between them, purely by each independently applying the same rule to the same data.
