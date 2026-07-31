# Lab 01 — Regtest network inspection

## Commands used

I created a Polar network named "Week 1 Bitcoin Fundamentals" with two Bitcoin Core
nodes and zero Lightning nodes, started it, then ran my implementation against node A:

```bash
cargo run -- lab01
```

`inspect_network` issues three RPCs in order. My host `bitcoin.conf` contains
`signet=1`, so I pass `-conf=/dev/null` to stop `bitcoin-cli` combining that with
`-regtest`:

```bash
bitcoin-cli -conf=/dev/null -regtest -rpcconnect=127.0.0.1 -rpcport=18443 \
  -rpcuser=polaruser -rpcpassword=polarpass getblockchaininfo
bitcoin-cli ... getblockcount
bitcoin-cli ... getbestblockhash
```

## Terminal output

```text
$ bitcoin-cli ... getblockchaininfo
  {
    "chain": "regtest",
    "blocks": 1,
    "bestblockhash": "7a14162c67d301c3b7deabcb63a1bcaee079c84f69227ff3171d010b0d76701a",
    "bits": "207fffff",
    "difficulty": 4.656542373906925e-10,
    "chainwork": "0000000000000000000000000000000000000000000000000000000000000004",
    "initialblockdownload": false
  }
$ bitcoin-cli ... getblockcount
  1
$ bitcoin-cli ... getbestblockhash
  7a14162c67d301c3b7deabcb63a1bcaee079c84f69227ff3171d010b0d76701a

--- NetworkSnapshot ---
{
  "chain": "regtest",
  "block_height": 1,
  "best_block_hash": "7a14162c67d301c3b7deabcb63a1bcaee079c84f69227ff3171d010b0d76701a"
}
```

The node answered every call, so it is running. The chain is `regtest`, height is 1, and
the best-block hash is `7a14162c…701a`. `inspect_network` refuses to build a snapshot
for any chain other than `regtest`, so getting a snapshot back is itself the check.

## Evidence references

`evidence/polar-network.png` shows the Polar canvas: the network named "Week 1 Bitcoin
Fundamentals", marked **Started**, containing two Bitcoin Core v30 nodes (`backend1` and
`backend2`) both running and peered to each other. The height badge in Polar's header is
the value cached when the network was created; the authoritative height at each point of
the assignment is in the run logs.

Full unedited run log is at `evidence/week1-labs-01-09.log`, lines 4-37, which is where
the output above is copied from. The container listing proving both Bitcoin Core nodes
were up is in `evidence/docker-ps.txt`.

## Explanation

These four pieces do genuinely different jobs, and I had been running them together in
my head before this lab.

**Bitcoin Core** is the node software itself. It validates blocks and transactions,
maintains the UTXO set and the mempool, and serves the RPC interface I called. Of the
four, it is the only one that knows any consensus rules.

**Docker** packages that node so it runs identically anywhere. Each node here is a
container with its own filesystem and data directory, which is exactly why two nodes can
run side by side without fighting over ports or a shared `.bitcoin` directory.

**Polar** is a GUI over Docker. It writes the compose configuration, assigns the port
mappings, wires the nodes to each other as peers, and gives me buttons instead of
commands. It implements no Bitcoin logic — if I deleted Polar the containers would keep
running fine.

**Regtest** is not software at all; it is a network mode inside Bitcoin Core. It gives a
private chain with a trivially easy difficulty target (`bits: 207fffff`, difficulty
`4.66e-10`), so a block can be mined instantly rather than by competing for real
hashpower, and blocks appear only when I ask for them via `generatetoaddress`. Addresses
carry the `bcrt1` prefix so regtest coins can never be mistaken for real ones.

Worth recording: the chain was already at height 1, not 0, before I did anything. Polar
mines one block while creating the network. That shifts every height in the rest of this
assignment up by one.
