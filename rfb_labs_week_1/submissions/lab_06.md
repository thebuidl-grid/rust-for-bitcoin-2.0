# Lab 06 — Transaction decoding

## Commands used

Rust:

```
cargo test --test lab_06
cargo fmt --check
cargo run --example lab06
```

`examples/lab06.rs` calls the completed `decode_verbose_transaction`, `input_outpoints`,
`identify_payment_and_change`, and `calculate_fee` functions against the real node, decoding the
Lab 05 payment TXID.

Bitcoin Core RPCs (run directly in Polar's node terminal):

```
bitcoin-cli getrawtransaction <txid> 2
```

Note: this had to be run *after* mining the transaction into a block (`bitcoin-cli
generatetoaddress 1 $MINER_ADDR`) — the first attempt, while the transaction was still unconfirmed,
returned no `prevout` data at all. `bitcoin-cli help getrawtransaction` confirms why: `prevout` is
"omitted if block undo data is not available," and undo data only exists for blocks that have
actually been mined. So despite the lab framing this as decoding "the unconfirmed transaction,"
this Bitcoin Core build requires the transaction to be confirmed first before verbosity 2 can
populate `prevout`.

## Terminal output

`cargo test --test lab_06`:

```
running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test decodes_inputs_outputs_and_virtual_size ... ok
test returns_consumed_outpoints ... ok
test distinguishes_receiver_output_from_change ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`cargo run --example lab06` (real node, via the completed Rust implementation):

```
DecodedTransaction {
    txid: "f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef",
    inputs: [
        DecodedInput {
            previous_output: OutPoint {
                txid: "c0d08e54a89d1060bc84618b9705d3e526d428c00c4cbcba309cdc5a57d9f182",
                vout: 0,
            },
            previous_value: 50.0,
        },
    ],
    outputs: [
        DecodedOutput { vout: 0, value: 1.0, address: Some("bcrt1q8xnsl28ymp70jxzmf7gnxx59aaa4tk6nl0cxs2"), script_pub_key_hex: "001439a70fa8e4d87cf9185b4f91331a85ef7b55db53" },
        DecodedOutput { vout: 1, value: 48.9999718, address: Some("bcrt1qrx2kpahveuhtcmujg45qtu9qcd09cywu8pgqg2"), script_pub_key_hex: "0014199560f6eccf2ebc6f92456805f0a0c35e5c11dc" },
    ],
    vsize: 141,
}
consumed outpoints: [ OutPoint { txid: "c0d08e54a89d1060bc84618b9705d3e526d428c00c4cbcba309cdc5a57d9f182", vout: 0 } ]
PaymentAndChange {
    payment: DecodedOutput { vout: 0, value: 1.0, address: Some("bcrt1q8xnsl28ymp70jxzmf7gnxx59aaa4tk6nl0cxs2"), .. },
    change: Some(DecodedOutput { vout: 1, value: 48.9999718, address: Some("bcrt1qrx2kpahveuhtcmujg45qtu9qcd09cywu8pgqg2"), .. }),
}
fee: 0.00002820000000269829
```

(The fee's trailing digits beyond `0.0000282` are ordinary `f64` floating-point imprecision from
subtracting two non-exact binary fractions — not a bug, and it matches Bitcoin Core's own reported
`"fee": 0.00002820` to the precision that matters.)

Raw `bitcoin-cli getrawtransaction <txid> 2` output (cross-checking the same transaction):

```
{
  "txid": "f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef",
  "vsize": 141,
  "vin": [
    {
      "txid": "c0d08e54a89d1060bc84618b9705d3e526d428c00c4cbcba309cdc5a57d9f182",
      "vout": 0,
      "prevout": { "value": 50.0, "scriptPubKey": { "address": "bcrt1q0tvlxqh4vkfzwuu9qun9d4txwrf76uj7syyvhy", ... } }
    }
  ],
  "vout": [
    { "value": 1.0, "n": 0, "scriptPubKey": { "address": "bcrt1q8xnsl28ymp70jxzmf7gnxx59aaa4tk6nl0cxs2", ... } },
    { "value": 48.9999718, "n": 1, "scriptPubKey": { "address": "bcrt1qrx2kpahveuhtcmujg45qtu9qcd09cywu8pgqg2", ... } }
  ],
  "fee": 0.0000282,
  "confirmations": 1
}
```

Value conservation, with actual numbers:

```
sum(inputs) = sum(payment outputs) + sum(change outputs) + fee
     50.0    =         1.0          +       48.9999718     + 0.0000282
     50.0    =                    50.0
```

## Evidence references

Terminal output above was captured directly from Polar's node terminal and from
`cargo run --example lab06`; no separate screenshots were taken for this lab.

## Explanation

Every inputs' total value must be accounted for entirely by this transaction's outputs plus the
fee — Bitcoin has no concept of value appearing or disappearing inside a transaction. The 50 BTC
consumed here splits into exactly two places: 1 BTC to the receiver (the actual payment) and
48.9999718 BTC back to a fresh address controlled by the sender's own wallet (the change output) —
that second output exists because the input UTXO (a whole 50 BTC coinbase reward) was far larger
than the 1 BTC actually being paid, and UTXOs can't be partially spent, only consumed whole and
reissued.

The fee isn't a line item anyone writes into the transaction — there's no `"fee"` output, no
address it's paid "to." It's simply **the leftover**: whatever value from the inputs isn't claimed
by any output is implicitly available for whichever miner includes this transaction in a block, and
Bitcoin Core computes it after the fact as `sum(inputs) - sum(outputs)`. This design is deliberate:
it means a transaction's fee is entirely determined by how its creator chose to split up the
inputs and outputs, and miners are naturally incentivized to prioritize transactions that leave a
bigger unclaimed remainder, without the protocol needing any separate fee-accounting mechanism at
all.
