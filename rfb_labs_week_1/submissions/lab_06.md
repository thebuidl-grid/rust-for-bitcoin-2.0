# Lab 06 — Transaction decoding

## Commands used

```bash
cargo test --test lab_06
bitcoin-cli -regtest getrawtransaction "<PAYMENT_TXID>" 2
```

## Terminal output

```text
TXID: [PASTE ACTUAL TXID]
Consumed outpoints and values: [PASTE EACH VIN TXID:VOUT AND PREVOUT VALUE]
Receiver payment output: [PASTE VOUT, ADDRESS, AND VALUE]
Change output: [PASTE VOUT, ADDRESS, AND VALUE]
Virtual size: [PASTE ACTUAL VSIZE]
Input total: [PASTE ACTUAL BTC TOTAL]
Output total: [PASTE ACTUAL BTC TOTAL]
Fee: [PASTE INPUT TOTAL - OUTPUT TOTAL]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- [Decoded transaction inputs, outputs, and virtual size](evidence/lab_06.png)

## Explanation

The decoded transaction demonstrates value conservation: the sum of input values equals the sum of payment and change outputs plus the fee. The fee is not represented by a special output. It is the value left unassigned when output values are subtracted from input values, and the miner may claim that difference in the block's coinbase transaction.
