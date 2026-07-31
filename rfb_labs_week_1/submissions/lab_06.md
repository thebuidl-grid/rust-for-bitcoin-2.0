# Lab 06 — Transaction decoding

## Commands used

```bash
cargo run -- lab06
```

```bash
bitcoin-cli ... getrawtransaction f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d 2
```

Verbosity `2` is meant to attach a `prevout` object to every input, which is what makes
the fee computable. Against a live node it did not, and I had to handle that — see the
explanation.

```bash
bitcoin-cli ... getrawtransaction 1c62ddb27fc72e5abefbc40dc882296e3772e57ee4df5a3a82e1b78554282c7f 1
```

## Terminal output

Decoded transaction:

```json
{
  "txid": "f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d",
  "inputs": [
    {
      "previous_output": { "txid": "1c62ddb27fc72e5abefbc40dc882296e3772e57ee4df5a3a82e1b78554282c7f", "vout": 0 },
      "previous_value": 50.0
    }
  ],
  "outputs": [
    {
      "vout": 0,
      "value": 1.0,
      "address": "bcrt1qga5wdzs456gvrk7kzh07lxxm5lxjarslkxm3m4",
      "script_pub_key_hex": "00144768e68a15a690c1dbd615dfef98dba7cd2e8e1f"
    },
    {
      "vout": 1,
      "value": 48.9999718,
      "address": "bcrt1qfydt8jgf2jadlql77rjnqhq6m567l4un0v2p2v",
      "script_pub_key_hex": "0014491ab3c90954badf83fef0e5305c1add35efd793"
    }
  ],
  "vsize": 141
}
```

Consumed outpoint: `1c62ddb27fc72e5abefbc40dc882296e3772e57ee4df5a3a82e1b78554282c7f:0`
— the same 50 BTC coinbase UTXO I inspected in Lab 04.

Payment versus change: `vout 0` pays 1.0 BTC to `bcrt1qga5wdzs…m3m4`, the receiver
address from Lab 02. `vout 1` returns 48.9999718 BTC to
`bcrt1qfydt8jgf2jadlql77rjnqhq6m567l4un0v2p2v`, an address that never appeared in my
commands — the miner wallet generated it internally as change.

```text
vsize = 141 vB
fee   = 0.0000282 BTC (20.00 sat/vB)

sum(inputs) = sum(payment outputs) + sum(change outputs) + fee
50 = 1 + 48.9999718 + 0.0000282
```

In satoshis, which is how I actually compute it:
`5000000000 = 100000000 + 4899997180 + 2820`.

## Evidence references

Full run log at `evidence/week1-labs-01-09.log`, lines 461-621, including the complete
verbose `getrawtransaction` output with `scriptSig`, `txinwitness`, and every
`scriptPubKey` field.

## Explanation

**Value conservation.** The single input brought in 50 BTC. The outputs assign
1 BTC to the receiver and 48.9999718 BTC back to the sender as change, totalling
48.9999718 + 1 = 49.9999718. The 0.0000282 BTC left over is the fee. Nothing is created
and nothing vanishes.

**Why the fee has no output of its own.** A transaction's outputs are an explicit list,
each naming an amount and a locking script. The fee is not in that list. It is the
*difference* between the total value the inputs bring in and the total the outputs assign
— value the transaction simply declines to allocate. Whoever mines the block collects it
by paying themselves that surplus in the block's coinbase transaction.

Structuring it as a residual rather than an output is what makes it enforceable. Consensus
already requires that outputs never exceed inputs, so the leftover is automatically
non-negative and automatically available to the miner. A dedicated "fee output" would need
a payee address at signing time, but the signer does not know which miner will win the
block. The residual sidesteps that entirely: the fee is paid to whoever mines it,
determined after the fact. It also means I cannot read a fee off the transaction alone —
I need the values of the outputs being spent, which live in earlier transactions. That is
the whole reason verbosity 2 exists.

**A practical wrinkle.** On Bitcoin Core v30 the `prevout` and `fee` fields are populated
from a block's *undo data*, which only exists once a transaction is in a block. My
transaction was still in the mempool, so verbosity 2 returned the inputs without
`prevout` and my first run failed with a missing-field error. I changed
`decode_verbose_transaction` to fall back to fetching the funding transaction by txid and
reading `vout[0].value` from it directly, which is where the 50.0 above comes from. This
works because Polar runs bitcoind with `txindex=1`. It also makes the earlier point
concrete: the fee genuinely is not carried in the transaction, so any tool that shows one
had to look up the inputs to find it.

**Change is not a refund.** `vout 1` exists because inputs must be spent whole. To pay
1 BTC from a 50 BTC output the transaction had to consume the entire 50 and hand back the
remainder. Omitting the change output would have donated 48.9999718 BTC to the miner as
fee.
