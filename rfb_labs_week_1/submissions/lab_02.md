# Lab 02 — Wallets and addresses

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). See `lab_01.md` for details.

## Commands used

```bash
# Create both wallets (node-wide call: the wallet is named in the parameters)
bitcoin-cli -regtest -datadir=$LAB/node-a createwallet miner
bitcoin-cli -regtest -datadir=$LAB/node-a createwallet receiver
bitcoin-cli -regtest -datadir=$LAB/node-a listwallets

# Wallet-scoped calls: each one needs -rpcwallet to say *which* wallet
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner    getnewaddress mining
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=receiver getnewaddress classmate
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner    getaddressinfo <mining-addr>
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=receiver getaddressinfo <mining-addr>
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=receiver getaddressinfo <classmate-addr>

# Rust implementation: lab02_wallets::{create_wallet, list_wallets,
# get_new_address, address_belongs_to_wallet}
cargo test --test lab_02
cargo run --example week1_walkthrough
```

## Terminal output

```text
========== Lab 02 — wallets and addresses ==========
loaded wallets   = ["miner", "receiver"]
mining address   = bcrt1q9j80atwfdpnk3k03l0006r3kzx8y7tere52thd
classmate addr   = bcrt1qdyv0w46hefge9tjgy5jelrnvn9hyp8lh4wae2j
mining addr is miner's    = true
mining addr is receiver's = false
classmate addr is receiver's = true
```

Three things are proven by that block:

1. `listwallets` returns both `miner` and `receiver`, so both are loaded.
2. Both addresses carry the `bcrt1…` prefix, which is the regtest bech32 HRP. A mainnet
   address would start `bc1…` and a testnet one `tb1…`, so the prefix alone rules out a
   wrong-network mistake.
3. The middle line is the important one: asking the **receiver** wallet about the
   **miner's** address returns `ismine = false`. Each address is claimed by exactly one
   wallet, and the wallets do not overlap.

## Evidence references

- Transcript section quoted above, produced by `cargo run --example week1_walkthrough`
  against the live node.
- Implementation: `src/labs/lab02_wallets.rs`. Note that `create_wallet` and
  `list_wallets` pass `None` for the wallet, while `get_new_address` and
  `address_belongs_to_wallet` pass `Some(wallet_name)` — the code makes the
  scoped/unscoped split explicit.
- Transport: `ProcessRpc::call` in `src/rpc.rs` turns that `Some(name)` into the
  `-rpcwallet=<name>` argument.
- Public tests: `cargo test --test lab_02` — 4 passed. Two of the four assert that the
  call carried the right wallet context, so a missing `-rpcwallet` fails the suite.
- No screenshots attached; the verbatim terminal output above is the evidence.

## Explanation

A Bitcoin Core node is one validating engine that can have many wallets loaded at once.
The chain state — blocks, the UTXO set, the mempool — is shared and global, so calls like
`getblockcount` or `getrawmempool` need no wallet at all. But keys, addresses, labels,
balances, and transaction history belong to a specific wallet, and the node has no way to
guess which one you meant.

`-rpcwallet=<name>` is what supplies that missing piece. It selects the wallet a
wallet-scoped RPC operates on, and it is required whenever more than one wallet is
loaded — Bitcoin Core will simply refuse the call rather than pick for you.

Getting it wrong is quiet and expensive, because the call usually still succeeds:

- `getnewaddress` in the wrong wallet hands you an address whose private key the other
  wallet holds. Coins sent there are not lost, but they are not yours to spend, which is
  the classic way lab payments "disappear".
- `getbalance` in the wrong wallet reports someone else's money, so you conclude a
  payment failed when it actually arrived.
- `sendtoaddress` in the wrong wallet spends the wrong wallet's UTXOs — on mainnet, real
  coins from an account you did not intend to touch.

That is also why `ismine` is the right ownership test. It is answered by the wallet you
addressed the question to, not by the chain, so it directly proves the mapping between
address and wallet rather than merely proving the address is well-formed.
