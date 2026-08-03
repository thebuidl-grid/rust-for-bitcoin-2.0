# Lab 06 — Transaction decoding

<!-- Replace every TODO line. The grader scores a section 0 while a TODO remains in it. Rewrite the Explanation in your own words. -->

## Commands used

```bash
# Verbosity 2 is required: it attaches each input's prevout, which carries the
# value being consumed. Verbosity 1 omits it and the fee cannot be derived.
bitcoin-cli getrawtransaction <txid> 2
```

Cross-check of the fee the wallet reports:

```bash
bitcoin-cli -rpcwallet=miner gettransaction <txid>
```

Tests:

```bash
cargo test --test lab_06
```

`decode_verbose_transaction` reads each `vin` (txid, vout, `prevout.value`) and each
`vout` (`n`, value, `scriptPubKey.hex`, optional address) plus `vsize`.
`calculate_fee` converts every amount to integer satoshis before subtracting, so the
result is exact rather than a floating-point approximation.

## Terminal output

A note on when this call works. Run against the transaction while it was still in the
mempool, verbosity 2 returned no `prevout` at all — the input showed only `scriptSig`
and `txinwitness`, and the fee could not be derived. Bitcoin Core attaches `prevout`
only once the transaction is in a block, so this decode was taken after Lab 07's
confirmation.

```
$ bitcoin-cli getrawtransaction 335c3feb471f8a50b354b8a4717fd53c81162922442fb3aef197de6ab5018d70 2
{
  "txid": "335c3feb471f8a50b354b8a4717fd53c81162922442fb3aef197de6ab5018d70",
  "hash": "efce2227d34099cacfabed6410f07ad8c982e7de4527635a5fb41144e1063bbb",
  "version": 2,
  "size": 222,
  "vsize": 141,
  "weight": 561,
  "locktime": 201,
  "vin": [
    {
      "txid": "5d6f8ed78981eeadd9d9f11e28f011233e6407d6e71eddbc6b18aaff10651ee9",
      "vout": 0,
      "prevout": {
        "generated": true,
        "height": 57,
        "value": 50.00000000,
        "scriptPubKey": {
          "hex": "0014f3afede355c75266b4c4dc794ac0c5155dcfe6c0",
          "address": "bcrt1q7wh7mc64cafxddxym3u54sx9z4wulekq06r04s",
          "type": "witness_v0_keyhash"
        }
      },
      "sequence": 4294967293
    }
  ],
  "vout": [
    {
      "value": 48.99997180,
      "n": 0,
      "scriptPubKey": {
        "hex": "001405abe474f44444a6d4e0eca27958620d9d1af315",
        "address": "bcrt1qqk47ga85g3z2d48qaj38jkrzpkw34uc4cslw2u",
        "type": "witness_v0_keyhash"
      }
    },
    {
      "value": 1.00000000,
      "n": 1,
      "scriptPubKey": {
        "hex": "00147ed3713923d8a8439d3366cf6d41a486763291a7",
        "address": "bcrt1q0mfhzwfrmz5y88fnvm8k6sdysemr9yd8qwznu7",
        "type": "witness_v0_keyhash"
      }
    }
  ],
  "fee": 0.00002820,
  "blockhash": "56af9a836f4f45c2e2fafff13f82f0ad15411097582f785da4d2bab261c36b1b",
  "confirmations": 1,
  "blocktime": 1785717745
}
```

**Inputs consumed.** One outpoint:

| Outpoint | Previous value |
| --- | --- |
| `5d6f8ed78981eeadd9d9f11e28f011233e6407d6e71eddbc6b18aaff10651ee9:0` | 50.00000000 BTC |

`generated: true` marks it as a coinbase output, mined at height 57 to the `mining`
address `bcrt1q7wh7mc64cafxddxym3u54sx9z4wulekq06r04s`.

**Outputs created.** Two:

| `n` | Value | Address | Role |
| --- | --- | --- | --- |
| 0 | 48.99997180 BTC | `bcrt1qqk47ga85g3z2d48qaj38jkrzpkw34uc4cslw2u` | change, back to the miner wallet |
| 1 | 1.00000000 BTC | `bcrt1q0mfhzwfrmz5y88fnvm8k6sdysemr9yd8qwznu7` | payment to `classmate` |

The payment output is identifiable because its address is the `classmate` address from
Lab 02. The change output went to an address neither wallet was asked for — the miner
wallet generated it internally.

**Virtual size:** 141 vbytes (222 raw bytes, 561 weight units).

**The equation**, in satoshis so nothing is lost to floating point:

```
sum(inputs)  = 5,000,000,000
payment      =   100,000,000
change       = 4,899,997,180
fee          =         2,820

5,000,000,000 = 100,000,000 + 4,899,997,180 + 2,820
5,000,000,000 = 5,000,000,000            ✓ both sides match
```

The fee of 2,820 satoshis is 0.00002820 BTC, exactly the `fee` the wallet reported in
Lab 05 before this transaction was ever confirmed. Two independent routes to the same
figure: subtracting the outputs from the input here, and the wallet's own bookkeeping
there. At 141 vbytes that works out to 20 sat/vB.

Note that no output anywhere holds the fee. It is the gap between what went in and
what came out, claimed by whoever mined block 202.

Tests:

```
$ cargo test --test lab_06
running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test returns_consumed_outpoints ... ok
test distinguishes_receiver_output_from_change ... ok
test decodes_inputs_outputs_and_virtual_size ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

![Verbose decode with prevout and derived fee](evidence/lab06-07-decode-and-confirmation.png)

A still from a screen recording of the `backend1` node terminal, holding the whole
decode in one frame: the input's `prevout` block with `"value": 50.00000000` and
`"generated": true`, both `vout` entries with their values and addresses, and
`"fee": 0.00002820` below them. The `blockhash` and `"confirmations": 1` at the foot
of the same output are the Lab 07 facts, captured in the same frame because both labs
read the same transaction.

## Explanation

A transaction consumes existing outputs and creates new ones. Each input names an
outpoint (`txid:vout`), and each output assigns an amount to a locking script.

Two of the outputs here do different jobs. One pays the receiver's address the
requested 1 BTC. The other returns the remainder to an address the sender controls
— the **change output**. Change is unavoidable, not a design choice: inputs are
consumed whole, so spending a 50 BTC UTXO to send 1 BTC must send the other ~49 BTC
somewhere, and the wallet sends it back to itself.

**The fee is the unassigned difference:**

```text
fee = sum(inputs) − sum(outputs)
```

There is no fee field and no fee output. A transaction never states its own fee.
Whatever value the inputs bring in and the outputs do not claim is implicitly
collected by the miner, who assigns it to themselves in the coinbase of the block
that includes this transaction.

This design is deliberate. Consensus enforces one rule — outputs may never exceed
inputs — and that single rule prevents inflation. A dedicated fee output would need
its own validation logic and would still have to be checked against the same
inequality, so it would add complexity while proving nothing extra. Leaving the fee
implicit means the fee is verified by the same arithmetic that already prevents
creating coins from nothing.

It also has a sharp practical consequence: **forgetting a change output does not
error, it donates.** A wallet that sends 1 BTC from a 50 BTC input and creates only
the payment output has, by definition, offered a 49 BTC fee. The transaction is
perfectly valid and the miner keeps the difference. Nothing in consensus flags this
as a mistake, which is why fee calculation is the wallet's responsibility.

This is also why `getrawtransaction` needs verbosity 2. The transaction lists which
outpoints it spends but not what they were worth — those values live in the earlier
transactions that created them. Without `prevout`, the input total is unknown and
the fee is uncomputable from the transaction alone.

Finally, `vsize` is virtual size in vbytes, the SegWit-aware measure of how much
block space the transaction occupies. Fee rates are quoted in satoshis per vbyte, so
`vsize` — not the byte length — is what determines the fee a transaction should
carry.
