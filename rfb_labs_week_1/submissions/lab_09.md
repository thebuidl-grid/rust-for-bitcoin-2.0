# Lab 09 — Multi-UTXO coin selection

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). See `lab_01.md` for details.

## Commands used

```bash
# Create Alice's wallet and a receiving address
bitcoin-cli -regtest -datadir=$LAB/node-a createwallet alice
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=alice getnewaddress alice-funding

# THREE separate 0.4 BTC payments — not one payment of 1.2 BTC
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner sendtoaddress <alice-addr> 0.4
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner sendtoaddress <alice-addr> 0.4
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner sendtoaddress <alice-addr> 0.4
bitcoin-cli -regtest -datadir=$LAB/node-a generatetoaddress 1 <mining-addr>

# Prove Alice owns three distinct UTXOs
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=alice listunspent

# Alice sends 1 BTC, which no single coin can cover
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=alice sendtoaddress <new-receiver-addr> 1
bitcoin-cli -regtest -datadir=$LAB/node-a getrawtransaction <spend-txid> 2

# Rust implementation: lab09_coin_selection::{create_three_funding_transactions,
# confirmed_utxos_for_address, send_combined_payment, audit_multi_utxo_spend}
cargo test --test lab_09
cargo run --example week1_walkthrough
```

## Terminal output

```text
========== Lab 09 — multi-UTXO coin selection ==========
alice address   = bcrt1q55tvqy4fz7ruks2wxz56a36m5xzh6k46l6er95
funding txids   = ["52fe3f4ea092e9c182e6291c9b06db8a0938738f929305e6ae8e63f03667cfd8",
                   "8d99a2b6a0b0cf952a95683bdc19e5d8524be87bd53acf3ad90b173d9b9f32b1",
                   "7bf9fdc3aa9c893a65dcd5c3909e746a2679608cc158316ac6a21e5cd5551d44"]
alice UTXO count = 3
  7bf9fdc3aa9c893a65dcd5c3909e746a2679608cc158316ac6a21e5cd5551d44:1 = 0.4 BTC
  8d99a2b6a0b0cf952a95683bdc19e5d8524be87bd53acf3ad90b173d9b9f32b1:0 = 0.4 BTC
  52fe3f4ea092e9c182e6291c9b06db8a0938738f929305e6ae8e63f03667cfd8:1 = 0.4 BTC
spend txid      = df6176248c04f1996ec2f6742d6f699bc3bce60b8af945e073984ea6efb500dd
inputs consumed = 3
payment         = 1 BTC to Some("bcrt1qevmmgdk57wfazg64kr7spv32qtgsj595dqqsrv")
change          = Some(0.1999724)
fee             = 0.0000276 BTC
```

**Alice owns three distinct UTXOs.** Three different txids, each 0.4 BTC. Note the `vout`
values differ (`:1`, `:0`, `:1`) — each funding transaction also created a change output
back to the miner, and Alice's output landed at a different index each time.

Each required claim about the spend:

- **More than one input was required** — `inputs consumed = 3`. Alice holds 1.2 BTC but
  her largest single coin is 0.4 BTC, so paying 1 BTC is impossible with one input, and
  even two inputs (0.8 BTC) fall short. Three is the minimum.
- **Selected inputs were consumed completely** — all three inputs total 1.2 BTC, and the
  transaction's outputs plus fee account for all 1.2 BTC. There is no residue left in the
  original coins; they no longer exist in the UTXO set.
- **The receiver received 1 BTC** — the payment output, to a fresh receiver address.
- **Surplus returned as change** — 0.1999724 BTC to a new address in Alice's wallet.
- **The difference is the fee** — 0.0000276 BTC (2760 satoshis).

**Value conservation:**

```text
sum(inputs) = payment + change + fee
0.4 + 0.4 + 0.4 = 1.0 + 0.1999724 + 0.0000276
      1.2       =            1.2
```

The fee is also instructive next to Lab 06. That transaction had one input and paid 1410
satoshis; this one has three inputs and pays 2760 — roughly double, for the same 1 BTC
payment. Fees are charged by transaction *size*, and each additional input adds bytes for
its outpoint, sequence, and witness. **More UTXOs means a bigger transaction means a
higher fee**, which is why wallets holding many small coins are expensive to spend from.

## Evidence references

- Transcript section quoted above from the live run.
- Implementation: `src/labs/lab09_coin_selection.rs`. It reuses Lab 04's `list_unspent`
  and Lab 06's `decode_verbose_transaction`, `identify_payment_and_change`, and
  `calculate_fee` — the audit is performed by the same code that was verified earlier.
- `confirmed_utxos_for_address` filters on `confirmations > 0` as well as the address, so
  an unconfirmed funding payment would not be counted as an owned coin.
- Coin selection itself is Bitcoin Core's, not ours. The lab does not choose the inputs;
  it *forces the situation* and then audits what the wallet did.
- Public tests: `cargo test --test lab_09` — 4 passed.
- No screenshots attached; the verbatim output above is the evidence.

## Explanation

**Why combining UTXOs reveals common ownership.** To spend a UTXO you must satisfy its
locking script, which means producing a valid signature for it. When a transaction spends
three inputs, whoever built it demonstrably held the keys for all three at the same
moment. That inference is available to anyone, forever, because the transaction is public
and permanent. This is the **common-input-ownership heuristic**, and it is the single
most reliable technique in chain analysis — not a probabilistic guess but a near-direct
consequence of how signing works.

The privacy trade-off is sharp, because it destroys separation that already existed. Look
at what was true *before* Alice's spend: three payments from three different transactions
to the same address. An observer could reasonably suspect a link but could not prove one.
After the spend, the three coins are provably one owner's. If those payments had come from
an employer, an exchange with Alice's identity documents, and a friend, then the moment
Alice combined them, the exchange's knowledge of who she is propagates to the other two.
One identified coin can taint an entire wallet's history retroactively.

Change makes it worse. The transaction has two outputs — 1 BTC and 0.1999724 BTC — and an
observer must guess which is the payment. Here it is easy: the "round" 1 BTC is a plausible
payment amount and the awkward remainder looks like change, an inference called the
**round-number heuristic**. Having identified the change output, the observer can follow
it into Alice's next transaction and continue tracking the wallet indefinitely.

The genuine trade-off is that consolidation is otherwise desirable:

- **Fees.** As the numbers above show, more inputs cost more. Consolidating when fees are
  low is a standard, sensible practice.
- **Dust.** Coins small enough that spending them costs more in fees than they are worth
  become permanently uneconomic to move.
- **Reliability.** A wallet that never combines coins may simply be unable to make a
  large payment, which is exactly Alice's position before the spend.

So the choice is fee efficiency against linkability, and it cannot be avoided by careful
spending alone — it is inherent to the UTXO model. Mitigations exist and all have costs:
coin control and labelling to keep coins from separate sources in separate accounts,
avoiding round-number payments, using a fresh address per receipt (which Bitcoin Core
already does for change), and collaborative transactions such as CoinJoin or PayJoin that
deliberately break the common-input assumption by having multiple parties contribute
inputs to one transaction.
