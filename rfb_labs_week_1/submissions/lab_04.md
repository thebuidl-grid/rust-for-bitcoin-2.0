# Lab 04 — UTXOs and outpoints

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). See `lab_01.md` for details.

## Commands used

```bash
# List the miner wallet's unspent outputs
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner listunspent

# The wallet balance to reconcile the independent sum against
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner getbalances

# Rust implementation: lab04_utxos::{list_unspent, select_spendable_utxo,
# outpoint, sum_spendable_utxos}
cargo test --test lab_04
cargo run --example week1_walkthrough
```

## Terminal output

```text
========== Lab 04 — UTXOs and outpoints ==========
wallet UTXO count = 1
txid          = a52e8ae89a6440c780007ff4be44c35a9429f09acf6de23622549ca0003d480c
vout          = 0
amount        = 50 BTC
confirmations = 101
address       = Some("bcrt1q9j80atwfdpnk3k03l0006r3kzx8y7tere52thd")
scriptPubKey  = 00142c8efeadc9686768d9f1fbdefd0e36118e4f2f23
spendable     = true
outpoint      = a52e8ae89a6440c780007ff4be44c35a9429f09acf6de23622549ca0003d480c:0
sum(spendable UTXOs) = 50 BTC
wallet trusted balance = 50 BTC
```

Every field the lab asks for is above: `txid`, `vout`, amount, confirmations, address,
locking script, and spendable state. The outpoint is the `txid:vout` pair on its own
line.

**Reconciliation.** `sum_spendable_utxos` walks the `listunspent` array itself and adds
up only the entries with `spendable = true`, arriving at 50 BTC. Bitcoin Core's own
`getbalances` reports `trusted = 50 BTC`. The two agree, and they agree *because they are
the same computation* — the balance is derived from the coins, not stored separately.

Note that `listunspent` returns only this single 50 BTC coin even though the wallet has
mined 101 blocks and holds 5050 BTC in total. The other 100 rewards are immature, so they
are not spendable outputs yet and do not appear. The locking script
`0014 2c8efe…2f23` is a P2WPKH script: `OP_0` followed by the 20-byte hash that the
`bcrt1q9j80at…` address encodes.

## Evidence references

- Transcript section quoted above from the live run.
- Implementation: `src/labs/lab04_utxos.rs`. `select_spendable_utxo` filters on
  `spendable` and picks the deepest coin, breaking ties on the outpoint so that repeated
  runs choose the same UTXO rather than whatever order the node happened to return.
- Model: `Utxo::outpoint()` in `src/model.rs` builds the `OutPoint`.
- Public tests: `cargo test --test lab_04` — 4 passed, including
  `sums_only_spendable_outputs`, which checks that non-spendable entries are excluded.
- No screenshots attached; the verbatim output above is the evidence.

## Explanation

Bitcoin has no accounts and stores no balances. What the network maintains is the **UTXO
set**: every transaction output that has been created and not yet spent. Each entry is a
discrete, indivisible coin with its own amount and its own locking script.

An **outpoint** is how a coin is named: the txid of the transaction that created it plus
the index of the output within that transaction, written `txid:vout`. The `vout` half is
essential because one transaction routinely creates several outputs, and the txid alone
cannot distinguish them. The outpoint is globally unique, and it is exactly what a future
transaction's input field contains — spending is pointing at an outpoint and satisfying
its locking script.

So a **wallet balance is not an account entry**. It is a derived figure, recomputed on
demand: the wallet scans the UTXO set for outputs whose scripts it can satisfy and adds
their amounts. Nothing anywhere holds the number "50 BTC" — that is why the independent
sum above matches, and it would be an odd coincidence if balances were stored separately.

Three practical consequences follow, all of which show up later in Week 1:

- **Coins are spent whole.** There is no way to spend part of a UTXO. Paying 1 BTC from a
  50 BTC coin means consuming all 50 and returning the remainder to yourself as change,
  which is exactly what Lab 06 decodes.
- **The composition of a balance matters, not just its size.** A wallet holding 1 BTC as
  a single coin and one holding it as a thousand tiny ones report the same balance but
  behave very differently — Lab 09 forces that difference deliberately.
- **A balance is a claim about spendability, not just ownership.** The `spendable` and
  `confirmations` fields are why Lab 03's 5050 BTC wallet could not send 1 BTC.
