# Lab 06 — Transaction decoding

## Commands used

Rust: `cargo run --example run` (calls `decode_verbose_transaction`, which runs):
- `getrawtransaction <txid> 2` — verbose decode with a `gettxout` fallback added for inputs where `prevout` wasn't embedded (see explanation)

## Terminal output

=== Lab 06: decode transaction ===
vsize: 141
inputs:
3d8cdd5803e49ad0eb24c9b7c1e1ae713e9cc37b40fd034fd3d05a31950cc820:0 value: 50
outputs:
vout 0: value 1 address Some("bcrt1qd0jasulp7k5xx3yrrg4s2jelnu526dj47qmskq") script 00146be5d873e1f5a86344831a2b054b3f9f28ad3655
vout 1: value 48.9999718 address Some("bcrt1qfcunzzure3375guu3d2muxv932zsdpseqf9r9y") script 00144e39310b83cc63ea239c8b55be19858a85068619
PaymentAndChange {
payment: DecodedOutput {
vout: 0,
value: 1.0,
address: Some("bcrt1qd0jasulp7k5xx3yrrg4s2jelnu526dj47qmskq"),
script_pub_key_hex: "00146be5d873e1f5a86344831a2b054b3f9f28ad3655",
},
change: Some(
DecodedOutput {
vout: 1,
value: 48.9999718,
address: Some("bcrt1qfcunzzure3375guu3d2muxv932zsdpseqf9r9y"),
script_pub_key_hex: "00144e39310b83cc63ea239c8b55be19858a85068619",
},
),
}
fee: 0.0000282
sum(inputs) 50 = sum(outputs) 49.9999718 + fee 0.0000282

## Evidence references

Screenshot: `evidence/lab06.png`

## Explanation

`getrawtransaction` with verbosity 2 decodes a raw, signed transaction back into readable fields — vin (inputs), vout (outputs), and size metrics like vsize (the transaction's weight-adjusted size, used for fee-rate calculations rather than raw byte size).

Value conservation: a Bitcoin transaction never destroys or creates value out of nothing — it can only spend existing UTXOs and produce new ones. My single 50 BTC input was split into two outputs: 1 BTC to the payment address, and 48.9999718 BTC returned as change to a new address controlled by the sender. Those two outputs don't sum back to exactly 50 — the 0.0000282 BTC difference is the transaction fee: `sum(inputs) = sum(outputs) + fee`, which my output confirms directly (50 = 49.9999718 + 0.0000282).

The fee has no dedicated output field or line item anywhere in the transaction — it's never explicitly stated. It's simply whatever value is left unclaimed between what the inputs supply and what the outputs claim. Miners are allowed to collect that leftover amount as an incentive for including the transaction in a block, which is exactly why sending a transaction with outputs summing to more than the inputs is invalid (negative fee), while under-claiming the inputs is completely legal.
