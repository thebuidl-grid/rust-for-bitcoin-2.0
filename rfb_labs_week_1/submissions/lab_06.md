# Lab 06 — Transaction decoding

## Commands used

1. **Decoding a verbose transaction**:
   ```bash
   bitcoin-cli getrawtransaction <txid> 2
   ```
   *(We use verbosity level `2` to display the values and scripts of the inputs (prevouts) directly, which is required to calculate the fee)*

2. **Running tests**:
   ```bash
   cargo test --test lab_06
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_06` returns:
```text
running 4 tests
test returns_consumed_outpoints ... ok
test distinguishes_receiver_output_from_change ... ok
test calculates_fee_from_input_and_output_values ... ok
test decodes_inputs_outputs_and_virtual_size ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Sample output of `getrawtransaction <txid> 2` (Mocked):
```json
{
  "txid": "payment-txid",
  "vsize": 141,
  "vin": [
    {
      "txid": "funding-txid",
      "vout": 0,
      "prevout": {
        "value": 1.50000000
      }
    }
  ],
  "vout": [
    {
      "value": 1.00000000,
      "n": 0,
      "scriptPubKey": {
        "hex": "0014aa",
        "address": "bcrt1qreceiver"
      }
    },
    {
      "value": 0.49999000,
      "n": 1,
      "scriptPubKey": {
        "hex": "0014bb",
        "address": "bcrt1qchange"
      }
    }
  ]
}
```

---

## Evidence references

- Code is implemented in [lab06_decode.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab06_decode.rs).
- Unit tests pass successfully, showing that we can extract `vin`, `vout`, calculate fees, and separate change outputs.

---

## Explanation

### Value Conservation
The values from the decoded transaction satisfy the conservation equation:
$$\sum \text{inputs} = \sum \text{payment outputs} + \sum \text{change outputs} + \text{fee}$$

Using the values from the output above:
$$1.5 = 1.0 + 0.49999 + 0.00001$$
$$1.5 = 1.5 \text{ BTC}$$

### Why the fee has no dedicated output
In Bitcoin transactions, transaction fees are **implicit**. There is no transaction output (vout) that explicitly says "pay $X$ satoshis to the miner".
Instead, the fee is calculated by miners as:
$$\text{Fee} = \sum \text{value of all inputs} - \sum \text{value of all outputs}$$
Any difference between the total input value and the total output value is left unallocated. Miners are allowed by the consensus rules to claim this difference as part of their coinbase transaction in the block they mine. 

Not having a dedicated fee output saves valuable block space (vsize) since creating an extra output would require more bytes, thereby increasing the transaction fee itself.
