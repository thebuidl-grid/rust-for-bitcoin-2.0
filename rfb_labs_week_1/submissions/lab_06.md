# Lab 06 — Transaction decoding

## Commands used

```
cargo test --test lab_06
cargo fmt --check
BITCOIN_CLI=<bitcoin-cli wrapper> cargo run --example lab06_demo
```

Underlying RPC (`src/labs/lab06_decode.rs`):
```
getrawtransaction cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe 2
```

**A real-node finding worth recording:** on this Bitcoin Core 30.0 build
(Polar's `polar-n1-backend1`, started with `-txindex=1`), calling
`getrawtransaction <txid> 2` while the transaction was *still unconfirmed*
returned `vin` entries **without** a `prevout` field, even though the RPC help
text says verbosity 2 includes "fee and prevout information." Re-running the
identical call after mining one confirming block populated `vin[].prevout`
and a top-level `fee` field correctly. In other words, on this node/version,
verbose prevout decoding is reliable for *confirmed* transactions (backed by
`-txindex`) but was not populated for this mempool-only transaction. So the
evidence below was captured immediately after mining the confirming block
(the same block Lab 07 uses), not before it — the lab's mock tests simulate
the idealized case where prevout is always present, but the live node
required confirmation first.

## Terminal output

`cargo test --test lab_06`:
```
running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test distinguishes_receiver_output_from_change ... ok
test decodes_inputs_outputs_and_virtual_size ... ok
test returns_consumed_outpoints ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab06_demo` against the live node:
```
DecodedTransaction {
    txid: "cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe",
    inputs: [
        DecodedInput {
            previous_output: OutPoint {
                txid: "8309b0a666fc79ec679cc77bc44d5ac3cda3962c27f991a7f35b4b8912f606bd",
                vout: 0,
            },
            previous_value: 50.0,
        },
    ],
    outputs: [
        DecodedOutput { vout: 0, value: 48.9999718,
            address: Some("bcrt1qv79z7a3ge4ckc7332ec6msrp80txv6xvhxvn63"),
            script_pub_key_hex: "0014678a2f7628cd716c7a315671adc0613bd66668cc" },
        DecodedOutput { vout: 1, value: 1.0,
            address: Some("bcrt1qxmst06mxnlgm5u7tscqsvvf892x8ulsasrl5ua"),
            script_pub_key_hex: "001436e0b7eb669fd1ba73cb86010631272a8c7e7e1d" },
    ],
    vsize: 141,
}

consumed outpoints = [OutPoint { txid: "8309b0a6...606bd", vout: 0 }]

payment_and_change = PaymentAndChange {
    payment: DecodedOutput { vout: 1, value: 1.0,
        address: Some("bcrt1qxmst06mxnlgm5u7tscqsvvf892x8ulsasrl5ua"), .. },
    change: Some(DecodedOutput { vout: 0, value: 48.9999718,
        address: Some("bcrt1qv79z7a3ge4ckc7332ec6msrp80txv6xvhxvn63"), .. }),
}

sum(inputs)  = 50
sum(outputs) = 49.9999718
fee          = 0.0000282
sum(inputs) == sum(outputs) + fee ? true
```

## Evidence references

- Screenshot: `submissions/evidence/Screenshot from 2026-08-01 13-58-25.png` — IDE
  terminal running `cargo test --test lab_06`, all 4 tests passing.
- Every consumed input: `8309b0a666fc79ec679cc77bc44d5ac3cda3962c27f991a7f35b4b8912f606bd:0`
  (the matured 50 BTC coinbase from Lab 04), worth `50.0` BTC.
- Every new output: `vout 0` = `48.9999718` BTC to
  `bcrt1qv79z7a3ge4ckc7332ec6msrp80txv6xvhxvn63` (change, back to the miner's
  own wallet); `vout 1` = `1.0` BTC to
  `bcrt1qxmst06mxnlgm5u7tscqsvvf892x8ulsasrl5ua` (the receiver's classmate
  address — the intended payment).
- `identify_payment_and_change` correctly separates these: `payment` = the 1
  BTC output to the receiver, `change` = the ~48.9999718 BTC output back to
  the sender.
- `vsize` = 141 (weight units / 4, matches the live node's own reported
  `vsize` for this tx).
- Value conservation, with real numbers:
  `50 = 1.0 (payment) + 48.9999718 (change) + 0.0000282 (fee)`.

## Explanation

The fee is never written as its own transaction output — it is simply
whatever value is left over after summing every declared output and
subtracting it from the sum of every consumed input
(`sum(inputs) - sum(outputs) = fee`, here `50 - 49.9999718 = 0.0000282`). A
dedicated "fee output" would require the sender to know in advance exactly
which miner will include the transaction and pay them directly, but Bitcoin
has no fixed miner identity to address a payment to — any miner might include
the transaction. Instead, the protocol defines the fee implicitly: whatever
value inputs exceed outputs by is claimed by whichever miner successfully
mines the block containing that transaction, added on top of the block
subsidy in the coinbase transaction. This also means a transaction's fee
cannot be hidden or disputed after the fact — it is fully determined and
independently verifiable by anyone who can see the inputs and outputs, exactly
as demonstrated by recomputing it here from the decoded transaction alone.
