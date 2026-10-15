# Lab 06 — Transaction decoding

> Environment: two local Bitcoin Core v30.2.0 regtest nodes started with `bitcoind`
> rather than Polar containers (Docker was unavailable). See `lab_01.md` for details.

## Commands used

```bash
# Decode the still-unconfirmed transaction from Lab 05
bitcoin-cli -regtest -datadir=$LAB/node-a getrawtransaction <txid> 2

# While the spend is only in the mempool, Bitcoin Core cannot attach `prevout`,
# so the value of each consumed output is read from the confirmed UTXO set:
bitcoin-cli -regtest -datadir=$LAB/node-a gettxout <prev-txid> <prev-vout> false

# Rust implementation: lab06_decode::{decode_verbose_transaction, input_outpoints,
# identify_payment_and_change, calculate_fee}
cargo test --test lab_06
cargo run --example week1_walkthrough
```

## Terminal output

```text
========== Lab 06 — decoding and value conservation ==========
txid  = b031668a7932c09fc4b775fa8c56e45afcc6617bb14cb5233a44f20e1dcb36ee
vsize = 141 vB
consumes a52e8ae89a6440c780007ff4be44c35a9429f09acf6de23622549ca0003d480c:0
input  a52e8ae89a6440c780007ff4be44c35a9429f09acf6de23622549ca0003d480c:0 = 50 BTC
output vout 0 = 48.9999859 BTC to Some("bcrt1qr6d08qa2ljxey3hu4e4mvuny895k46hvz26wq8")
output vout 1 = 1 BTC to Some("bcrt1qdyv0w46hefge9tjgy5jelrnvn9hyp8lh4wae2j")
payment = 1 BTC (vout 1)
change  = 48.9999859 BTC
fee     = 0.0000141 BTC
conservation: 50 = 1 + 48.9999859 + 0.0000141
```

Identifying each part:

- **Consumed outpoint:** `a52e8ae8…:0` — the single 50 BTC coinbase UTXO from Lab 04.
- **Payment output:** `vout 1`, 1 BTC, to `bcrt1qdyv0w46…`, the receiver's `classmate`
  address from Lab 02.
- **Change output:** `vout 0`, 48.9999859 BTC, to `bcrt1qr6d08qa…` — a fresh address
  that never appeared in any earlier lab, generated internally by the miner wallet.
- **Virtual size:** 141 vB.
- **Miner fee:** 0.0000141 BTC (1410 satoshis), which matches the `fee: -1.41e-5` the
  sending wallet reported in Lab 05.

**Value conservation, with actual values:**

```text
sum(inputs) = sum(payment outputs) + sum(change outputs) + fee
   50.0      =        1.0           +      48.9999859     + 0.0000141
   50.0      =                        50.0
```

Note that the payment is at `vout 1` and the change at `vout 0`. Bitcoin Core shuffles
output order deliberately, so output position tells you nothing — the payment has to be
identified by its address, which is what `identify_payment_and_change` does.

## Evidence references

- Transcript section quoted above from the live run.
- Implementation: `src/labs/lab06_decode.rs`.
- **A real-node finding worth recording.** `getrawtransaction … 2` attaches each input's
  `prevout` only for *mined* transactions, because those values come from the block's
  undo data. This lab decodes a transaction that is still in the mempool, so `prevout` is
  absent and the raw response cannot price the inputs at all. `decode_input` therefore
  falls back to `gettxout <txid> <vout> false`. The `false` matters: the transaction
  being decoded already spends that output, so a mempool-aware lookup reports it as gone,
  while the confirmed-only view still has it.
- `calculate_fee` converts to whole satoshis before subtracting. Summing BTC as
  floating point leaves the fee a fraction of a satoshi off, and a fee is an exact
  integer quantity.
- Public tests: `cargo test --test lab_06` — 4 passed.
- No screenshots attached; the verbatim output above is the evidence.

## Explanation

**Why the fee has no output of its own.** A Bitcoin transaction lists inputs and outputs.
The inputs authorise the consumption of specific existing UTXOs; the outputs create new
ones. Consensus requires only that the outputs do not create more value than the inputs
consume. Whatever the sender leaves unassigned — the difference between the two totals —
is the fee, and it is claimed by the miner in the coinbase output of the block that
includes the transaction.

So the fee is not paid *to* anyone by the transaction; it is simply value the transaction
declines to allocate. This is genuinely elegant:

- **It cannot be forged.** There is no "fee" field for a wallet to lie about. Any node
  recomputes it from the input and output values, which is exactly what `calculate_fee`
  does above. A dedicated fee output, by contrast, would be one more thing to validate
  and one more place for the stated value and real value to disagree.
- **It needs no address.** The miner is not known when the transaction is built. Paying
  the fee to a specific miner's address would require guessing who will mine it.
- **It works out naturally.** Because outputs cannot exceed inputs, a transaction that
  simply forgets to add a change output pays the entire remainder as fee. This is not a
  hypothetical — it is a well-known way people have lost large sums.

**Why change is unavoidable.** UTXOs are indivisible. The wallet had one 50 BTC coin and
needed to pay 1 BTC, and there is no operation for spending part of a coin. It must
consume all 50 BTC and create a new output paying the remainder back to itself. This is
also why the change goes to a *fresh* address rather than back to the mining address:
reusing an address would publicly link the change to the sender, so wallets derive a new
one each time. Lab 09 shows the limits of that defence.

**Why verbosity matters.** The serialized transaction contains only *pointers* to the
outputs it spends — `txid:vout` — never their values. The amounts live in the earlier
transactions that created them. This is why auditing value conservation requires either
verbosity 2 or an explicit lookup of each previous output: `sum(inputs)` is not a number
the transaction states about itself. That is also the deeper reason nodes maintain a UTXO
set at all.
