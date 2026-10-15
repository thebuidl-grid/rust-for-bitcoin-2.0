# Lab 05 — Broadcast and mempool

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). See `lab_01.md` for details.

## Commands used

```bash
# Send exactly 1 BTC from the miner wallet — and do NOT mine afterwards
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner \
    sendtoaddress <classmate-addr> 1

# Is the TXID sitting in this node's local mempool?
bitcoin-cli -regtest -datadir=$LAB/node-a getrawmempool

# The sender's view of its own transaction
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=miner gettransaction <txid>

# The receiver's view of the incoming money
bitcoin-cli -regtest -datadir=$LAB/node-a -rpcwallet=receiver getbalances

# Rust implementation: lab05_mempool::{send_btc, get_raw_mempool,
# get_transaction_status, observe_unconfirmed_payment}
cargo test --test lab_05
cargo run --example week1_walkthrough
```

## Terminal output

```text
========== Lab 05 — broadcast and mempool state ==========
txid               = b031668a7932c09fc4b775fa8c56e45afcc6617bb14cb5233a44f20e1dcb36ee
in local mempool   = true
sender status      = WalletTransactionStatus { txid: "b031668a7932c09fc4b775fa8c56e45afcc6617bb14cb5233a44f20e1dcb36ee", confirmations: 0, amount: -1.0, fee: Some(-1.41e-5), block_hash: None }
receiver balances  = WalletBalances { trusted: 0.0, untrusted_pending: 1.0, immature: 0.0 }
```

Each of the four required claims, and the line that proves it:

- **The TXID is in the node's local mempool** — `in local mempool = true`, from
  `getrawmempool` containing `b031668a…`.
- **The sender reports zero confirmations** — `confirmations: 0`, and `block_hash: None`
  because no block contains it yet.
- **The receiver sees an untrusted-pending balance** — `untrusted_pending: 1.0` with
  `trusted: 0.0`. The receiver's wallet knows about the money and still will not treat it
  as spendable.
- **Broadcast is not confirmation** — the transaction exists, is fully valid, is signed,
  has a TXID, and has been relayed, yet `trusted` is still `0.0` on the receiving side.

The sender's `amount: -1.0` is negative because `gettransaction` reports the effect on
*that* wallet, and the miner wallet is 1 BTC worse off. The `fee: -1.41e-5` is 1410
satoshis, shown negative for the same reason and known only to the sending wallet.

## Evidence references

- Transcript section quoted above from the live run.
- Implementation: `src/labs/lab05_mempool.rs`. `observe_unconfirmed_payment` performs the
  send and then three read-only observations in order, with no mining call anywhere in
  the function — the unconfirmed state is captured, not manufactured.
- `WalletTransactionStatus::confirmations` is an `i64`, not a `u64`, because a conflicted
  transaction reports a *negative* depth. `block_hash` and `fee` are `Option` because
  neither exists for an unconfirmed transaction sent by another wallet.
- Public tests: `cargo test --test lab_05` — 4 passed.
- No screenshots attached; the verbatim output above is the evidence.

## Explanation

A payment passes through four distinct states, and conflating any two of them is how
double-spend losses happen.

**1. Built and signed.** The wallet selects UTXOs, builds outputs (payment plus change),
computes a fee, and signs each input. At this moment the transaction is a complete, valid
object with a fixed TXID — and it exists *only on the sending machine*. Nobody else knows
it exists. It can still be discarded without a trace.

**2. Broadcast.** The transaction is handed to a node and relayed to peers. This is a
networking event, nothing more. It proves the transaction was *published*; it proves
nothing about acceptance.

**3. In the mempool.** Each node independently validates the transaction — signatures,
that the inputs exist and are unspent, that it meets policy rules like minimum relay fee —
and if it passes, holds it in memory as a candidate for the next block. The mempool is
**per node and not consensus**. Nodes can and do disagree about its contents; a node that
restarts loses its mempool; a transaction can be evicted for a low fee or replaced by a
higher-fee conflict. This is exactly the state captured above: `getrawmempool` on *this*
node contains the TXID, which says nothing about what any other node holds.

**4. Confirmed.** A miner includes the transaction in a valid block. Only now does it
become part of the agreed history, and only now do all nodes converge on its existence.

The receiver's `untrusted_pending` label names the gap precisely. The wallet has seen the
transaction and believes it is valid, but it is *untrusted* because nothing yet prevents
it from being replaced or simply never mined. The sender could broadcast a conflicting
transaction spending the same input to themselves with a higher fee; miners have every
incentive to take the higher fee. Until a block commits to it, the payment is a proposal.

This is the practical rule the lab is teaching: **a TXID is a receipt for a request, not
a receipt for a payment.** Accepting a TXID as proof of payment is the mistake; waiting
for confirmations, which Lab 07 and Lab 08 quantify, is the fix.
