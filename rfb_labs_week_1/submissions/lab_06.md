# Lab 06 — Transaction decoding

## Commands used

```bash
cargo test --test lab_06
bitcoin-cli -regtest getrawtransaction <payment-txid> 2
```

## Terminal output

The verbose decoded transaction exposed each `vin` with previous `txid:vout` and `prevout.value`, every `vout` with value/address/script, and the transaction `vsize`. The receiver output paid exactly 1 BTC, the remaining wallet-controlled output was change, and the fee was calculated as `sum(inputs) - sum(outputs)`.

## Evidence references

Evidence is the Lab 06 test run and the verbose raw transaction output. The value equation recorded was: total input value equals receiver payment plus change plus miner fee.

## Explanation

Bitcoin transactions do not have account debits and credits. They consume whole previous outputs and create new outputs. Any input value not assigned to a new output is the miner fee, so the fee is an implicit difference rather than a dedicated transaction output.
