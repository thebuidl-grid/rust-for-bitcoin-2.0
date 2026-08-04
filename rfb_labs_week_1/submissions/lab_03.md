
# Lab 03 — Coinbase maturity

## Commands used

Rust:

```
cargo test --test lab_03
cargo fmt --check
cargo run --example lab03
```

`examples/lab03.rs` calls the completed `demonstrate_coinbase_maturity` function against the real
node. It creates a fresh `miner2` wallet (with no other balance) first — the pre-existing `miner`
wallet from Lab 02 already had spendable funds by this point, which would let the "premature"
payment succeed from those other funds instead of correctly failing, defeating the point of the
proof.

Bitcoin Core RPCs (run directly in Polar's node terminal, on the `miner`/`receiver` wallets, before
running the Rust example):

```
bitcoin-cli generatetoaddress 1 $MINER_ADDR
bitcoin-cli getblockcount
bitcoin-cli -rpcwallet=miner getbalances
bitcoin-cli -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1
bitcoin-cli generatetoaddress 100 $MINER_ADDR
bitcoin-cli getblockcount
bitcoin-cli -rpcwallet=miner getbalances
```

## Terminal output

`cargo test --test lab_03`:

```
running 4 tests
test preserves_insufficient_funds_error ... ok
test reads_nested_wallet_balances ... ok
test demonstrates_first_coinbase_becoming_spendable_at_height_101 ... ok
test mines_requested_number_of_blocks ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`cargo run --example lab03` (real node, fresh `miner2` wallet, via the completed Rust
implementation):

```
CoinbaseMaturityReport {
    height_after_first_block: 103,
    balance_after_first_block: WalletBalances {
        trusted: 0.0,
        untrusted_pending: 0.0,
        immature: 50.0,
    },
    premature_spend_error: "error code: -6\nerror message:\nInsufficient funds",
    final_height: 203,
    final_balance: WalletBalances {
        trusted: 50.0,
        untrusted_pending: 0.0,
        immature: 3650.0,
    },
}
```

Raw `bitcoin-cli` output on the `miner` wallet, run directly in Polar's terminal beforehand
(cross-checking the same behavior — this node already had 1 block mined from Lab 01, so heights
are offset by one from a fresh chain, but the maturity rule is identical):

```
$ bitcoin-cli generatetoaddress 1 $MINER_ADDR
[ "2d7dc74f46973d058990cd78b61ba707e33d3d06018e33b22bdaee2b488eb453" ]

$ bitcoin-cli getblockcount
2

$ bitcoin-cli -rpcwallet=miner getbalances
{
  "mine": { "trusted": 0.0, "untrusted_pending": 0.0, "immature": 50.0 },
  ...
}

$ bitcoin-cli -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1
error code: -6
error message:
Insufficient funds

$ bitcoin-cli generatetoaddress 100 $MINER_ADDR
[ ... 100 block hashes ... ]

$ bitcoin-cli getblockcount
102

$ bitcoin-cli -rpcwallet=miner getbalances
{
  "mine": { "trusted": 50.0, "untrusted_pending": 0.0, "immature": 5000.0 },
  ...
}
```

Both the Rust implementation and the raw `bitcoin-cli` calls agree: right after mining, the reward
is `immature` and unspendable (attempting to spend it fails with "Insufficient funds" even though
the wallet's total balance shows the coins exist); after 100 further confirmations, that first
reward moves into `trusted` and becomes spendable, while every reward from the more-recent blocks
remains `immature`.

## Evidence references

Terminal output above was captured directly from Polar's node terminal and from
`cargo run --example lab03`; no separate screenshots were taken for this lab.

## Explanation

Bitcoin Core enforces `COINBASE_MATURITY = 100`: a coinbase output (the block reward + fees paid to
whoever mined a block) cannot be spent until it has at least 100 confirmations, i.e. 100 additional
blocks have been mined on top of the block that created it. This is a consensus rule, not just a
wallet-display quirk — any transaction trying to spend an immature coinbase output would be
rejected by every node on the network, not just the local wallet software.

The rule exists because a freshly-mined block can still be **reorganized out** of the chain if a
competing, heavier chain appears (see Lab 10). If coinbase rewards were spendable immediately, a
reorg could retroactively invalidate the source of coins that had already been spent onward to
other people, effectively letting a miner counterfeit money during a brief chain split. Waiting 100
blocks makes that reorg-and-double-spend scenario astronomically expensive to pull off in practice,
by which point the reward is considered safely settled.

The lab conventionally mines 101 blocks on a fresh chain (rather than 100) because the *first*
block itself doesn't count toward its own maturity — you need 100 blocks mined **after** it, so the
chain must reach height 101 (1 + 100) before that very first reward unlocks. On this node the chain
wasn't perfectly fresh (it already had prior blocks from earlier labs), so the exact heights differ
from the "1 → 101" example, but the underlying rule — spendable only 100 confirmations after the
block that created it — is exactly what both the raw RPC calls and the Rust implementation proved.
