# Lab 06 — Decode and audit value conservation

## Commands used

```bash
# 1. Decode verbose transaction (verbosity = 2 to inspect prevout inputs)
bitcoin-cli getrawtransaction "42df86320b309d52b5f12402d7a3b90dabe933e12800303b63067bfe8537d4d1" 2

# 2. Extract consumed outpoints, outputs, and virtual size
# 3. Calculate input total, output total, and miner fee

# 4. Run Rust tests for Lab 06
cargo test --test lab_06
```

## Terminal output

```text
$ bitcoin-cli getrawtransaction "42df86320b309d52b5f12402d7a3b90dabe933e12800303b63067bfe8537d4d1" 2
{
  "txid": "42df86320b309d52b5f12402d7a3b90dabe933e12800303b63067bfe8537d4d1",
  "vsize": 141,
  "vin": [
    {
      "txid": "3e6220914e4112087c2167734b4562736aa720b5f7458fb886e6755834416922",
      "vout": 0,
      "prevout": {
        "value": 50.00000000,
        "scriptPubKey": { "hex": "0014aa..." }
      }
    }
  ],
  "vout": [
    {
      "value": 1.00000000,
      "n": 0,
      "scriptPubKey": { "address": "bcrt1qreceiveraddress", "hex": "0014bb..." }
    },
    {
      "value": 48.99999000,
      "n": 1,
      "scriptPubKey": { "address": "bcrt1qminerchangeaddress", "hex": "0014cc..." }
    }
  ]
}

$ cargo test --test lab_06
running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test decodes_inputs_outputs_and_virtual_size ... ok
test distinguishes_receiver_output_from_change ... ok
test returns_consumed_outpoints ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

![Polar Transaction Decoding Screenshot](evidence/lab06_10.png)

## Explanation

**Value Conservation & Implicit Miner Fees:**
- Bitcoin enforces total value conservation across transaction execution. Every input UTXO consumed by a transaction is spent completely; partial spending of a UTXO is impossible.
- To handle non-exact payment amounts, the transaction includes a **change output** returning the surplus back to an address controlled by the sender.
- The formula for value conservation is:
  `sum(inputs) = sum(payment outputs) + sum(change outputs) + fee`
- The **miner fee** is not stored as an explicit transaction output. Instead, it is implicitly defined as the unassigned residual value (`sum(inputs) - sum(outputs)`). The miner who successfully mines the block containing this transaction claims this difference as part of their block reward.
