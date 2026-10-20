# Lab 06 — Decode and audit value conservation

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_06

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest getrawtransaction "payment-txid" 2
```

## Terminal output

```json
// verbose output from getrawtransaction:
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

## Evidence references

- Verified via Rust test suite `tests/lab_06.rs` functions.
- Checked transaction details and calculated fee in Polar node explorer view.

## Explanation

- **Why the fee is the unassigned difference**: In Bitcoin transactions, the fee is not explicitly specified as an output field. Instead, it is implicitly defined as the difference between the total value of the inputs consumed and the total value of the outputs created:
  `Fee = Sum(Inputs) - Sum(Outputs)`
  This implicit design saves block space and simplifies validation. Because consensus rules dictate that a transaction cannot create more value than it consumes, the unallocated difference is automatically allowed to be claimed by the miner as a reward in their coinbase transaction.
