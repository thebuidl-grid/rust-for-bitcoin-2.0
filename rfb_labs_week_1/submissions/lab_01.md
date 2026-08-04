# Lab 01 — Regtest network inspection

## Commands used

Rust:

```
cargo test --test lab_01
cargo fmt --check
cargo run --example lab01
```

`examples/lab01.rs` calls the completed `inspect_network` function against the real node. Since
`bitcoin-cli` is only installed inside Polar's Docker container (not on the host), the example
builds a `ProcessRpc` that runs commands through `docker exec -u bitcoin polar-n1-backend1
bitcoin-cli -regtest ...` instead of calling `bitcoin-cli` directly — the same thing Polar's own
node terminal does under the hood.

Bitcoin Core RPCs (run directly in Polar's node terminal to confirm the raw data independently):

```
bitcoin-cli getblockchaininfo
bitcoin-cli getblockcount
bitcoin-cli getbestblockhash
```

## Terminal output

`cargo test --test lab_01`:

```
running 4 tests
test builds_verified_network_snapshot ... ok
test reads_block_height ... ok
test reads_regtest_chain ... ok
test reads_best_block_hash ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo run --example lab01` (real node, via the completed Rust implementation):

```
NetworkSnapshot {
    chain: "regtest",
    block_height: 1,
    best_block_hash: "333cdea629a079d28f2ee8a1a10b2f95a21f5c04763df8af98a65445b3362732",
}
```

`bitcoin-cli getblockchaininfo` (run directly in Polar's terminal, cross-checking the same node):

```
{
  "chain": "regtest",
  "blocks": 1,
  "headers": 1,
  "bestblockhash": "333cdea629a079d28f2ee8a1a10b2f95a21f5c04763df8af98a65445b3362732",
  ...
}
```

`bitcoin-cli getblockcount` / `bitcoin-cli getbestblockhash`:

```
1
333cdea629a079d28f2ee8a1a10b2f95a21f5c04763df8af98a65445b3362732
```

The Rust implementation's output matches the raw `bitcoin-cli` output exactly: chain `regtest`,
height `1`, and the same best-block hash.

## Evidence references

- `submissions/evidence/lab_01/network-started.png` — Polar network "Week 1 Bitcoin Fundamentals"
  running, Bitcoin Core node `backend1`, height 1.
- `submissions/evidence/lab_01/rpc-output.png` — `bitcoin-cli getblockcount` and
  `bitcoin-cli getbestblockhash` run directly inside the node's Polar terminal.

## Explanation

- **Docker** provides an isolated container to run Bitcoin Core in, with its own filesystem and
  network namespace, so it doesn't touch anything else installed on the host machine and can be
  torn down and recreated cleanly.
- **Polar** is a desktop app that manages Docker Compose configurations for you: it generates the
  container definitions, starts/stops them, and gives you a ready-made terminal wired up with the
  right `-regtest` flag and RPC credentials, so you don't have to hand-configure `bitcoin.conf` or
  remember connection details yourself.
- **Bitcoin Core** (`bitcoind`) is the actual node software — it implements the Bitcoin protocol,
  maintains the local copy of the blockchain, exposes the RPC server that `bitcoin-cli` and my
  Rust code both talk to, and can hold a wallet.
- **regtest** ("regression test mode") is a private chain mode built into Bitcoin Core: blocks can
  be mined instantly on demand instead of waiting for real proof-of-work, coins have no real value,
  and the network is fully isolated from mainnet and testnet — which makes it safe and fast to
  practice wallet, transaction, and mining operations against.
