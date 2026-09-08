# Lab 01 — Regtest network inspection

## Commands used

Polar setup:
- Created network **Week 1 Bitcoin Fundamentals** in Polar, added one Bitcoin Core
  node (`backend1`, running `polarlightning/bitcoind:30.0`), and started it (Docker
  container `polar-n1-backend1`).

Bitcoin Core RPCs called by the Rust implementation (`inspect_network` in
`src/labs/lab01_network.rs`), run from the node's terminal via `bitcoin-cli`:
- `getblockchaininfo` (to read `chain`)
- `getblockcount` (to read the current height)
- `getbestblockhash` (to read the current tip hash)

Rust commands:
```
cargo test --test lab_01
cargo fmt --check
cargo run --example lab01_demo   # runs inspect_network() against the live node
```

Raw `bitcoin-cli` equivalent used to cross-check the Rust output:
```
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash
```

## Terminal output

`cargo test --test lab_01`:
```
running 4 tests
test builds_verified_network_snapshot ... ok
test reads_best_block_hash ... ok
test reads_block_height ... ok
test reads_regtest_chain ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab01_demo` against the live Polar node (the Rust
`inspect_network` function, driving real `bitcoin-cli` calls):
```
chain            = regtest
block_height     = 2
best_block_hash  = 5e75102f9e1c616e23dc4bd7b0fbbe8a2a218d4110a6776eeeb481c78026a708
```

Direct `bitcoin-cli getblockchaininfo` on the node (trimmed) confirms the same
values:
```
{
  "chain": "regtest",
  "blocks": 2,
  "bestblockhash": "5e75102f9e1c616e23dc4bd7b0fbbe8a2a218d4110a6776eeeb481c78026a708",
  "bits": "207fffff",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000006"
}
```

The node reports `chain = regtest`, a live height of 2 blocks, and the matching
best-block hash — proving `inspect_network` correctly reads and verifies a
running regtest node rather than any other network.

## Evidence references

- Screenshot: `submissions/evidence/Screenshot from 2026-08-01 13-57-53.png` — IDE
  terminal running `cargo test --test lab_01`, all 4 tests passing.
- Docker: `docker ps` shows container `polar-n1-backend1` (image
  `polarlightning/bitcoind:30.0`) `Up`, with regtest P2P/RPC ports mapped
  (`18444->18443`, `19445->18444`), confirming Polar started the node.
- Rust: `cargo run --example lab01_demo` output above was produced by the
  actual `inspect_network()` function in this repository calling `bitcoin-cli`
  against that running container — not a mocked value — via a thin
  `bitcoin-cli` wrapper pointed at the container's RPC.
- Cross-check: the raw `bitcoin-cli getblockchaininfo` / `getblockcount` /
  `getbestblockhash` calls above return the same chain, height, and hash the
  Rust snapshot reports, so the two sources agree.

## Explanation

**Docker** is the container runtime Polar uses under the hood. Instead of
installing Bitcoin Core (or LND, etc.) directly on the host, Polar launches
each node as an isolated Docker container with its own filesystem, process
space, and networking — so multiple nodes with different versions can run
side by side without clashing, and everything can be torn down cleanly by
removing the containers.

**Polar** is the desktop app / orchestration layer on top of Docker. It lets
you visually design a Bitcoin network (drag in nodes, wire up peer
connections), then translates that design into Docker Compose services,
starts/stops the containers, and exposes conveniences like a built-in
terminal per node so you can run `bitcoin-cli` directly inside the container
Polar created.

**Bitcoin Core** is the actual node software running inside that container —
the full implementation of the Bitcoin protocol (validation, mempool, wallet,
P2P, and the JSON-RPC server that `bitcoin-cli` talks to). Polar and Docker
are just the delivery mechanism; Bitcoin Core is what does the real work of
maintaining consensus state and answering RPCs like `getblockchaininfo`.

**Regtest** ("regression test mode") is one of Bitcoin Core's network types
(alongside mainnet, testnet, and signet). It's a private, on-demand chain with
trivial proof-of-work (`bits = 207fffff`), so blocks can be mined instantly on
command via `generatetoaddress` instead of waiting for real mining — ideal for
fast, deterministic, disposable test networks with fake, worthless coins. This
lab's `inspect_network` explicitly rejects any chain whose `getblockchaininfo`
`chain` field isn't `"regtest"`, precisely because code and coins that behave
correctly on mainnet/testnet must never be assumed to be running on a private
test chain, and vice versa.
