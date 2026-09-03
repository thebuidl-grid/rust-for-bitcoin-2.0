# Lab 06 — Transaction decoding

## Commands used

Ran `getrawtransaction b0be49f7ba206557fe14510f6d6f00e16a9cf1d7155fb459babd97695a8b7fe0 2` and compared its input with `listunspent`.

## Terminal output

Input `23dddf...ab2a:0` was 50 BTC; vsize was 141 vB. Output 0 paid receiver 1 BTC and output 1 returned 48.99997180 BTC. Therefore `50 = 1 + 48.99997180 + 0.00002820` BTC.

## Evidence references

Verbose transaction JSON captured both output scripts and addresses; the earlier confirmed UTXO JSON captured the consumed input value.

## Explanation

Input value equals every new output plus the fee. The fee is the unassigned input-output difference; miners claim it through their block's coinbase transaction, not a dedicated output.
