# Lab 06 — Decode transaction

## Commands used

```
cargo test --test lab_06
bitcoin-cli -regtest getrawtransaction "<txid>" 2
```

*RPC is the one issued by `decode_verbose_transaction` in `src/labs/lab06_decode.rs`; `input_outpoints`, `identify_payment_and_change`, and `calculate_fee` operate on the decoded transaction locally (no further RPCs), verified against the mocked RPC client in `tests/lab_06.rs`. Run the `bitcoin-cli` line against your live Polar regtest node to capture the terminal output below.*

## Terminal output

Decoded the transaction from Lab 05 twice: first while it was still unconfirmed (mempool-only), then again after Lab 07 mined it.

**While unconfirmed** (immediately after Lab 05's `sendtoaddress`, no block mined yet):

```
$ bitcoin-cli -regtest getrawtransaction "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2" 2
{
  "txid": "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2",
  "size": 222,
  "vsize": 141,
  "vin": [
    {
      "txid": "c313884361a36760928891af56c5dfb4cca60bf7bea888640528e90e5cbac1ec",
      "vout": 0,
      "scriptSig": { "asm": "", "hex": "" },
      "txinwitness": [ "3044...", "032aaf..." ],
      "sequence": 4294967293
    }
  ],
  "vout": [
    { "value": 1.00000000, "n": 0, "scriptPubKey": { "address": "bcrt1qx208aadpjxz7ftargmdy64amhslmycnjll2xxl", ... } },
    { "value": 48.99997180, "n": 1, "scriptPubKey": { "address": "bcrt1qx40ga8jdkclfmfxxtrtr2yfljmckuxr5ttte9a", ... } }
  ]
}
```

Note there's **no `prevout` field on `vin` here**. This is a genuine finding, not an assumption: `decode_verbose_transaction` (`src/labs/lab06_decode.rs`) requires `prevout.value` on every input and would fail with `MissingField("prevout")` against this exact output.

**After confirmation** (re-run once Lab 07 mined the block), the same call *does* include `prevout`:

```
$ bitcoin-cli -regtest getrawtransaction "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2" 2
{
  ...
  "vin": [
    {
      "txid": "c313884361a36760928891af56c5dfb4cca60bf7bea888640528e90e5cbac1ec",
      "vout": 0,
      "prevout": {
        "generated": true,
        "height": 1,
        "value": 50.0,
        "scriptPubKey": { "address": "bcrt1qtdwur5a220ta9f0lndtdqf45f0kmaplt3xea5l", ... }
      },
      ...
    }
  ],
  ...
}
```

So on Bitcoin Core v30.0.0, `prevout` is only populated for verbosity-2 `getrawtransaction` once the referencing transaction is confirmed and its input's block context is known — not for pure mempool transactions. `decode_verbose_transaction` only works correctly against confirmed transactions on this node.

One input (the 50 BTC block-1 coinbase spent in Lab 05), two outputs: 1 BTC to the receiver, 48.99997180 BTC back to the sender's own change address. `vsize` is 141 vbytes. Fee = `sum(inputs) − sum(outputs)` = `50.00000000 − (1.00000000 + 48.99997180)` = **0.00002820 BTC**, matching the `fee` field `gettransaction` reported in Lab 05.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a bitcoin-cli ...` against a real regtest node (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

**Value conservation:** every satoshi entering a transaction (via its inputs) must be accounted for by what leaves it (its outputs, plus the fee). Nothing can be created or silently destroyed. For this transaction:

```
input:   50.00000000  (the block-1 coinbase UTXO being spent)
outputs:  1.00000000  (payment to the receiver)
        + 48.99997180 (change back to the sender)
        -----------
        = 49.99997180
fee:     50.00000000 − 49.99997180 = 0.00002820 BTC
```

Input total exactly equals output total plus fee — `50.00000000 = 49.99997180 + 0.00002820`. That equality is the proof: nothing was invented or lost, the arithmetic simply balances.

**Why the fee has no dedicated output:** a Bitcoin transaction never states its fee directly — there's no `"fee"` field in the raw transaction data itself (the `fee` shown by `gettransaction` in Lab 05 is the *wallet* computing and reporting it for convenience, not something stored on-chain). The fee is *implicit*: it's whatever value the inputs supply that the outputs don't claim. A miner including the transaction in a block is entitled to keep that leftover amount as an extra reward on top of the block subsidy. This is also exactly why `calculate_fee` (`src/labs/lab06_decode.rs`) doesn't read a "fee" field from the decoded transaction — it computes `sum(inputs) − sum(outputs)`, deriving the fee the same way the network does.
