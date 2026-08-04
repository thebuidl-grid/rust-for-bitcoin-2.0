# Lab 01 — Regtest network inspection

> **Environment note.** Docker was not available on this machine, so instead of a Polar
> network I ran two Bitcoin Core v30.2.0 nodes directly in regtest mode (`node-a` on
> p2p 18444 / rpc 18443, `node-b` on p2p 18455 / rpc 18454). Every RPC, wallet, and
> block below comes from those live nodes. The Rust code is unchanged and drives them
> through `bitcoin-cli` exactly as it would drive Polar's Bitcoin Core containers.

## Commands used

```bash
# Start the regtest node (Polar equivalent: create the network and press Start)
bitcoind -datadir=$LAB/node-a -daemon

# Raw RPCs the Rust implementation issues, run by hand to cross-check it
bitcoin-cli -regtest -datadir=$LAB/node-a getblockchaininfo
bitcoin-cli -regtest -datadir=$LAB/node-a getblockcount
bitcoin-cli -regtest -datadir=$LAB/node-a getbestblockhash

# Rust implementation: lab01_network::{get_chain, get_block_height,
# get_best_block_hash, inspect_network}
cargo test --test lab_01
RFB_NODE_A="-regtest -datadir=$LAB/node-a" cargo run --example week1_walkthrough
```

## Terminal output

`inspect_network` against the freshly started node:

```text
========== Lab 01 — regtest network inspection ==========
chain            = regtest
block height     = 0
best block hash  = 0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206
```

The same three facts straight from `bitcoin-cli`, confirming the Rust output:

```text
$ bitcoin-cli -regtest -datadir=$LAB/node-a getblockchaininfo
{
  "chain": "regtest",
  "blocks": 0,
  "headers": 0,
  "bestblockhash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
  ...
}
```

`0f9188f1…` is the regtest genesis block hash, which is itself proof that the node is
on regtest and not on mainnet or testnet.

## Evidence references

- Full ten-lab transcript produced by `cargo run --example week1_walkthrough`; the
  Lab 01 section is quoted above in full.
- Driver source: `examples/week1_walkthrough.rs` (Lab 01 section).
- Implementation: `src/labs/lab01_network.rs`. `inspect_network` refuses to continue
  unless `chain == "regtest"`, so the height and hash above cannot have come from any
  other network.
- Public tests: `cargo test --test lab_01` — 4 passed.
- No screenshots are attached; the node was driven from a terminal, and the verbatim
  command output above is the evidence.

## Explanation

The four pieces do different jobs and are easy to confuse because they arrive together.

**Bitcoin Core** is the actual Bitcoin node software. It validates blocks and
transactions, keeps the UTXO set, holds the mempool, serves the RPC interface, and (in
these labs) also runs the wallet. It is the only component here that knows the consensus
rules. Everything else is scaffolding around it.

**regtest** is a chain mode inside Bitcoin Core, not a separate program. It gives you a
private chain with its own genesis block, its own address prefix (`bcrt1…`), and a
difficulty target so low that a block is found instantly. Crucially, blocks are mined on
demand with `generatetoaddress` rather than by a real proof-of-work race, so the whole
of Week 1 can be reproduced in seconds. The coins are worthless by construction, which
is what makes it safe to experiment.

**Docker** packages and isolates. Each node runs in its own container with its own
filesystem, its own data directory, and its own ports, so several Bitcoin Core versions
can run side by side without colliding and can be thrown away cleanly.

**Polar** is the orchestration and GUI layer on top of Docker. It composes the network —
which nodes exist, which versions they run, how they are wired to each other — starts and
stops them together, and gives each one a terminal. Polar does not implement any Bitcoin
logic; remove it and the same nodes still work, you just have to wire them yourself.

That last point is exactly what this run demonstrates. Because Docker was unavailable I
did Polar's job by hand: two data directories, two config files, two ports, one
`addnode` to link them. The RPC surface the Rust code talks to is identical, which is
the real lesson — the node is the system of record, and Polar and Docker are convenience.
