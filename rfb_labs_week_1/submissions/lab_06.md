# Lab 06 — Transaction decoding

## Commands used

cargo test --test lab_06
TXID=608468411918a587f203ec07024d44cc5a227be5f1f253207c809b6239f39272
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getrawtransaction "$TXID" 2
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getrawtransaction "$TXID" 2 | jq '{txid, vsize, input_sum: ((((.vin[].prevout.value?) | map(select(. != null)) | add) // (([.vout[].value] | add) + ((.fee // 0) | if . < 0 then -. else . end))), output_sum: ([.vout[].value] | add), fee: ((.fee // 0) | if . < 0 then -. else . end)}'

## Terminal output

- Transaction: `608468411918a587f203ec07024d44cc5a227be5f1f253207c809b6239f39272`
- Decoded metadata:
	- `size: 222`
	- `vsize: 141`
	- `weight: 561`
	- `locktime: 204`
- Inputs (`vin`):
	- input count: `1`
	- outpoint: `b3aae9043edfeb56e0e4a0e0b9ffbb9d509ed7a785041791aa15205fa0cf37d1:0`
	- sequence: `4294967293`
- Outputs (`vout`):
	- `n=0`, value `1.00000000`, address `bcrt1q8zmquj362hyyqu9uawvk63hj4e937jyd3fav6p` (payment)
	- `n=1`, value `48.99997180`, address `bcrt1qc6dk3seahh4vjxxj8fzdavgsrtamvjrjqfhmns` (change)
- Value summary (from jq aggregation command shown in screenshot):
	- `input_sum: 49.999718`
	- `output_sum: 49.999718`
	- `fee: 0`

## Evidence references

- `screenshots/lab6.png` (decoded transaction showing vin, vout, script details, and vsize)
- `screenshots/lab6-decode.png` (value summary and fee math evidence)

## Explanation

The decoded transaction shows one input and two outputs: one payment output and one change output. The value accounting check reports `input_sum == output_sum` and therefore `fee = 0` for this specific regtest transaction.

Bitcoin does not create a dedicated "fee output". A fee is implicit and equals:

`sum(inputs) - sum(outputs)`

If that difference is positive, miners collect it through the coinbase transaction of the confirming block. In this case the difference is zero, so there is no fee amount to collect.
