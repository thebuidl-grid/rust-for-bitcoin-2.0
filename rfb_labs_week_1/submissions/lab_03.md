# Lab 03 — Coinbase maturity

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). See `lab_01.md` for details.

## Commands used

```bash
# 1. Mine a single block to the miner address
bitcoin-cli -regtest -datadir=$LAB/node-a generatetoaddress 1 <mining-addr>
bitcoin-cli -regtest -datadir=$LAB/node-a getblockcount
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner getbalances

# 2. Try to spend the reward too early — this is expected to FAIL
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner \
    sendtoaddress <classmate-addr> 1

# 3. Mine 100 more blocks and re-read height and balances
bitcoin-cli -regtest -datadir=$LAB/node-a generatetoaddress 100 <mining-addr>
bitcoin-cli -regtest -datadir=$LAB/node-a getblockcount
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner getbalances

# Rust implementation: lab03_maturity::{mine_blocks, get_balances,
# attempt_payment, demonstrate_coinbase_maturity}
cargo test --test lab_03
cargo run --example week1_walkthrough
```

## Terminal output

```text
========== Lab 03 — coinbase maturity ==========
height after 1 block = 1
balances after 1     = WalletBalances { trusted: 0.0, untrusted_pending: 0.0, immature: 50.0 }
premature spend error= error code: -6
error message:
Insufficient funds
final height         = 101
final balances       = WalletBalances { trusted: 50.0, untrusted_pending: 0.0, immature: 5000.0 }
```

Reading that against the three things the lab asks to prove:

| Claim | Evidence |
| --- | --- |
| Chain reached height 101 | `final height = 101` |
| First coinbase is now spendable | `trusted` moved from `0.0` to `50.0` BTC |
| Later rewards remain immature | `immature = 5000.0` BTC, i.e. the 100 newest rewards |

At height 1 the wallet holds 50 BTC that it can see but cannot touch: `immature = 50.0`
while `trusted = 0.0`. The premature spend therefore fails with RPC error `-6`,
`Insufficient funds` — Bitcoin Core's own wording, preserved rather than paraphrased.
The arithmetic at height 101 also checks out: 101 blocks × 50 BTC = 5050 BTC total, of
which 50 BTC is spendable and the remaining 5000 BTC is still maturing.

## Evidence references

- Transcript section quoted above from the live run.
- Implementation: `src/labs/lab03_maturity.rs`. `demonstrate_coinbase_maturity` captures
  the RPC error text into `CoinbaseMaturityReport::premature_spend_error` instead of
  swallowing it, and deliberately returns an error if the premature spend *succeeds* —
  a silent success would mean the demonstration did not actually demonstrate anything.
- Public tests: `cargo test --test lab_03` — 4 passed, including
  `preserves_insufficient_funds_error`.
- No screenshots attached; the verbatim output above is the evidence.

## Explanation

A coinbase output — the block subsidy plus fees paid to whoever mined the block — cannot
be spent until **100 further blocks** have been built on top of the block that created
it. That is the `COINBASE_MATURITY = 100` consensus rule, and it is enforced by every
node, not by the wallet. A transaction spending a coinbase output younger than 100
confirmations is invalid and will be rejected outright.

The rule exists because of reorganizations. Ordinary transactions survive a reorg: if a
block is orphaned, its transactions usually return to the mempool and get mined again.
Coinbase outputs cannot do that, because a coinbase is bound to the specific block that
created it. If the block is orphaned, its reward stops existing. Without a maturity
window, a miner could spend a fresh reward, have a competing branch win, and leave those
coins spent from an output that no longer exists — invalidating every downstream
transaction. Burying the reward under 100 blocks makes a reorg deep enough to undo it
prohibitively expensive.

That is why the labs mine **101** blocks on a fresh chain and not 100. After block 1 the
reward from block 1 has one confirmation. It needs 100 blocks *on top* of it, so it
becomes spendable when the chain reaches height 101 — the moment block 101 exists. Mining
exactly 101 blocks is the shortest path to a wallet with real spendable balance, which is
what the rest of Week 1 needs. Every reward from blocks 2–101 is still immature at that
point, which is precisely the 5000 BTC shown above.

Note also what the failed spend teaches about `getbalances`: `trusted`, `immature`, and
`untrusted_pending` are three genuinely different states, and only `trusted` is money you
can spend right now. A wallet that reported one merged number would have made the error
above look like a bug.
