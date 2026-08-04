# Lab 03 — Coinbase maturity

## Commands used

Rust commands, run against the live Polar node (`polar-n1-backend1`, chain
height 2 going in, from Lab 02):
```
cargo test --test lab_03
cargo fmt --check
BITCOIN_CLI=<bitcoin-cli wrapper> cargo run --example lab03_demo
```

`demonstrate_coinbase_maturity` (in `src/labs/lab03_maturity.rs`) drives these
`bitcoin-cli` RPCs in order:
```
generatetoaddress 1 bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv   # mine to miner's "mining" address
getblockcount
getbalances                                    -rpcwallet=miner
sendtoaddress bcrt1qxmst06m... 1               -rpcwallet=miner   # expected to fail: immature
generatetoaddress 100 bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv
getblockcount
getbalances                                    -rpcwallet=miner
```

Cross-check directly on the node: `bitcoin-cli -rpcwallet=miner getbalances`.

## Terminal output

`cargo test --test lab_03`:
```
running 4 tests
test mines_requested_number_of_blocks ... ok
test demonstrates_first_coinbase_becoming_spendable_at_height_101 ... ok
test preserves_insufficient_funds_error ... ok
test reads_nested_wallet_balances ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab03_demo` against the live node (chain was already at
height 2 from Lab 02's setup, so heights are offset by 2 from a fresh chain,
but the +100-block maturity gap is identical):
```
CoinbaseMaturityReport {
    height_after_first_block: 3,
    balance_after_first_block: WalletBalances {
        trusted: 0.0,
        untrusted_pending: 0.0,
        immature: 50.0,
    },
    premature_spend_error: "error code: -6\nerror message:\nInsufficient funds",
    final_height: 103,
    final_balance: WalletBalances {
        trusted: 50.0,
        untrusted_pending: 0.0,
        immature: 5000.0,
    },
}
```

Direct `bitcoin-cli -rpcwallet=miner getbalances` after the run confirms the
same final numbers:
```
{
  "mine": {
    "trusted": 50.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 5000.00000000
  },
  "lastprocessedblock": { "hash": "6342ab2c...", "height": 103 }
}
```

## Evidence references

- Screenshot: `submissions/evidence/Screenshot from 2026-08-01 13-57-53.png` — IDE
  terminal running `cargo test --test lab_03`, all 4 tests passing.
- After mining one block (height 3), `trusted = 0` and `immature = 50` — the
  50 BTC subsidy exists but cannot be spent yet.
- The immediate 1 BTC send attempt returns Bitcoin Core error code `-6`,
  `"Insufficient funds"` — proven by `premature_spend_error` capturing the
  exact RPC error text rather than swallowing it.
- After mining 100 additional blocks (final height = 103 = 3 + 100), the
  *first* coinbase reward (50 BTC) becomes `trusted`, while all 100 rewards
  mined afterward are still `immature` (5000 BTC = 100 × 50 BTC), matching
  Bitcoin Core's live `getbalances` output exactly.

## Explanation

`COINBASE_MATURITY = 100` is a Bitcoin Core consensus rule stating a coinbase
output (a block's mining subsidy + fees) cannot be spent until it has at least
100 confirmations — i.e. until 100 additional blocks have been built on top of
the block containing it. This exists to protect against chain reorganizations:
if a spend of an immature coinbase were allowed and that block were later
orphaned by a reorg, the spent coins would never have existed on the winning
chain, retroactively invalidating any transaction built on top of them. Waiting
100 blocks makes a reorg deep enough to un-mature a coinbase reward
astronomically expensive in proof-of-work.

The lab conventionally mines 101 blocks on top of a *fresh* (height-0) chain
because the first block mined (height 1) needs exactly 100 more blocks stacked
on it to reach 100 confirmations, so height 101 is the first height at which
the height-1 coinbase is spendable. In this run the chain was not fresh (it
already sat at height 2 from Lab 02), so the same *relative* rule shows up as
height 3 → height 103 (a 100-block gap) instead of 1 → 101, but the underlying
maturity math is identical: spendability begins exactly 100 blocks after the
coinbase was mined.
