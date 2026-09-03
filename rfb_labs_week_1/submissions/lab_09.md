# Lab 09 — Multi-UTXO coin selection

## Commands used

Created wallet `alice`, generated an address, sent three 0.4 BTC payments, mined one block, ran `listunspent`, sent 1 BTC, and decoded it with `getrawtransaction ... 2`.

## Terminal output

Alice had confirmed UTXOs `a6d3ffad...235d:0`, `2b20c3fe...12fa:0`, and `7e9e8043...059c:1`, each 0.4 BTC. Spend `7ee383637e1fec806c2e2b07a8c0ab35d07b8a61a5f2bbccfdc47305a493f0a2` has three inputs, pays 1 BTC, returns 0.19994480 BTC change, and pays 0.00005520 BTC fee.

## Evidence references

`listunspent` showed the three confirmed funding outputs. Verbose spending JSON showed three vin entries, vsize 276 vB, payment output, and change output.

## Explanation

Three 0.4 BTC inputs were necessary to fund 1 BTC plus fee; surplus became change and the input-output difference became fee. Combining inputs exposes likely common ownership, which is a privacy trade-off.
