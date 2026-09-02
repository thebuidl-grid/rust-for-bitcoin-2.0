# Lab 10 — Competing branches and reorganization

## Commands used

Both Bitcoin Core nodes come from the same Polar network, addressed on different RPC
ports (node A on 18443, node B on 18444):

```bash
export BITCOIN_CLI_ARGS='-conf=/dev/null -regtest -rpcconnect=127.0.0.1 -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass'
export NODE_B_CLI_ARGS='-conf=/dev/null -regtest -rpcconnect=127.0.0.1 -rpcport=18444 -rpcuser=polaruser -rpcpassword=polarpass'
cargo run -- lab10
```

Record the common tip on both, then split:

```bash
bitcoin-cli -rpcport=18443 ... getblockchaininfo
bitcoin-cli -rpcport=18444 ... getblockchaininfo

bitcoin-cli -rpcport=18443 ... getpeerinfo
bitcoin-cli -rpcport=18443 ... disconnectnode 172.22.0.2:43486
bitcoin-cli -rpcport=18443 ... disconnectnode backend2
bitcoin-cli -rpcport=18444 ... disconnectnode 172.22.0.3:50468
bitcoin-cli -rpcport=18444 ... disconnectnode backend1
```

Polar has each node `addnode` the other, so there are **two** connections between them —
one inbound, one outbound. Disconnecting only one leaves the nodes talking and no fork
forms. My runner drops every peer returned by `getpeerinfo` and then re-checks that both
report zero peers before mining anything.

Mine privately on each side, then reconnect:

```bash
bitcoin-cli -rpcport=18443 ... generatetoaddress 2 bcrt1qsx0ep7fjmjyu5gsp88hr3q9av9muwlp59jluem
bitcoin-cli -rpcport=18444 ... generatetoaddress 4 bcrt1qwcr6lp6fjh4qg648jz8vncjqj9r0r9qxnxp4gn

bitcoin-cli -rpcport=18443 ... addnode backend2:18444 onetry
```

`addnode` needs the address node B *listens* on, not the ephemeral inbound address from
`getpeerinfo`.

## Terminal output

Common tip before the split — both nodes agree:

```json
node A: { "height": 110, "best_block_hash": "4d3fbe42caa7f427858c9f2bd1aa71a4e3ae806efe198abd7ba75943ddb30056",
          "chainwork": "...000000de" }
node B: { "height": 110, "best_block_hash": "4d3fbe42caa7f427858c9f2bd1aa71a4e3ae806efe198abd7ba75943ddb30056",
          "chainwork": "...000000de" }
```

```text
node A is isolated
node B is isolated
```

Competing private tips after mining 2 blocks on A and 4 on B:

```json
{
  "node_a": {
    "height": 112,
    "best_block_hash": "4ec073a618efdd0852ccb7c4fc3082aa782af7f9ff4d38128b002cbca1601f0d",
    "chainwork": "00000000000000000000000000000000000000000000000000000000000000e2"
  },
  "node_b": {
    "height": 114,
    "best_block_hash": "51b766f54aa7fabc8253038cdab73aa1a4ce7a00aed9d8dfb739a7d488f0714f",
    "chainwork": "00000000000000000000000000000000000000000000000000000000000000e6"
  }
}
```

Two genuinely different chains: different tips, different heights, chainwork `…e2` versus
`…e6`.

After `addnode backend2:18444 onetry`, both nodes converged:

```json
{
  "common_tip_before_split": "4d3fbe42caa7f427858c9f2bd1aa71a4e3ae806efe198abd7ba75943ddb30056",
  "final_tips": {
    "node_a": { "height": 114, "best_block_hash": "51b766f54aa7fabc8253038cdab73aa1a4ce7a00aed9d8dfb739a7d488f0714f",
                "chainwork": "...000000e6" },
    "node_b": { "height": 114, "best_block_hash": "51b766f54aa7fabc8253038cdab73aa1a4ce7a00aed9d8dfb739a7d488f0714f",
                "chainwork": "...000000e6" }
  },
  "converged": true
}
```

Node A abandoned its own tip `4ec073a6…1f0d` and adopted node B's `51b766f5…714f`.

## Evidence references

`evidence/polar-network.png` shows the two-node topology this lab depends on: `backend1`
and `backend2` on one Polar network with a peer link between them, which is the connection
the lab breaks and then restores.

Full run log at `evidence/week1-lab-10.log`, 392 lines covering the whole sequence:
tips before the split (lines 47-59), the disconnects and isolation checks (216-233),
private mining on both nodes (255-260), competing tips (306-318), the reconnect (320),
and the final report (361-391).

## Explanation

**Why one branch went stale.** While the nodes were isolated, each extended the shared
history at `4d3fbe42…0056` independently. Neither did anything wrong — both branches were
internally valid, both had real proof of work, and each node was entirely correct to build
on what it knew. Two valid chains existed simultaneously. That is normal, not a fault
condition, and it happens on mainnet whenever two miners find a block at nearly the same
moment.

The branches could not both survive because they spend from the same ancestor. When the
nodes reconnected and exchanged headers, node A saw a branch with more accumulated work
than its own and switched. Its two blocks — `4ec073a6…1f0d` and its parent — became
**stale**: valid blocks that are no longer part of the best chain. Their coinbase rewards
are gone, which is exactly the scenario Lab 03's maturity rule protects against. Any
ordinary transactions that had been in them would return to the mempool and could be mined
again, but a coinbase cannot, because it was minted by that specific block.

**What a reorganization is.** Node A disconnected its own two blocks, rolled its UTXO set
back to the fork point at height 110, and then connected node B's four blocks, replaying
their transactions. A reorg of depth 2. Nothing was deleted from disk — node A still has
the stale blocks — but they no longer form the active chain.

**Why most work wins.** Node B's chain won on `chainwork` `…e6` versus node A's `…e2`.
Chainwork is the cumulative expected number of hashes across every block, so it is a
measure of energy actually expended, not a count of blocks. On this regtest chain every
block has identical difficulty so more blocks meant more work, but the rule is about work,
not length — a shorter chain of higher-difficulty blocks would beat a longer chain of
easier ones.

The alternatives are all worse, and this is the heart of it:

- **Arrival time** would reward whoever is best-connected, and would need a trusted clock.
  There is no global time in a distributed system, and timestamps are attacker-controlled.
- **Miner identity** would require a registry of privileged miners, which reintroduces the
  central authority Bitcoin exists to remove.
- **Social claim** — whoever asserts loudest, or a vote among participants — is free to
  fake, so an attacker could just spin up nodes and outvote everyone.

Accumulated work is the only one of these that cannot be forged. Claiming a chain requires
having actually burned the electricity, and any node can verify that independently from
the headers alone, with no trusted third party and no communication. Node A did not consult
anyone, take a vote, or check who mined what. It compared two numbers it computed itself
and switched.

**The limit, which matters.** Most-work only selects among **valid** chains. Node A
verified every one of node B's four blocks before adopting them. Had any contained an
invalid transaction, node A would have rejected the whole branch and kept its own shorter
chain regardless of how much work backed the alternative. Proof of work decides *which*
valid history everyone agrees on; it never makes an invalid transaction valid.
