# Lab 06 — Transaction decoding

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getrawtransaction 7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152 2
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest gettxout 352808782139e14c47df525c7186b2f6d4ee8632e69b6a2fd77fde63f02d0011 0 false
jq -n '{input_sats:5000000000,payment_sats:100000000,change_sats:4899997180,fee_sats:(5000000000-100000000-4899997180),balanced:(5000000000==(100000000+4899997180+(5000000000-100000000-4899997180)))}'
```

## Terminal output

```text
txid: 7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152
vsize: 141
vin: 352808782139e14c47df525c7186b2f6d4ee8632e69b6a2fd77fde63f02d0011:0
previous value (gettxout with mempool excluded): 50.00000000 BTC

vout 0: 1.00000000 BTC -> bcrt1qduu2p93zkvp0v69uqglj0dlaam0rwplhk6gf3u
  script: 00146f38a09622b302f668bc023f27b7fdeede3707f7
vout 1: 48.99997180 BTC -> bcrt1qx7fvtam77m707p43e75q3aa2emzgyvxw9zg7lm
  script: 00143792c5f77ef6fcff06b1cfa808f7aacec48230ce

input_sats:  5,000,000,000
payment_sats:  100,000,000
change_sats:  4,899,997,180
fee_sats:             2,820 (0.00002820 BTC)
balanced: true
```

## Evidence references

Live verbose decode plus a separate previous-output query and exact integer-satoshi
calculation. Core 30 omitted `vin.prevout` for this mempool transaction (its help permits
that when undo data is unavailable), so `gettxout ... false` supplied the real confirmed
input value without treating the mempool spend as removal.

## Explanation

The transaction spent a 5,000,000,000-sat input. It created a 100,000,000-sat payment and
a 4,899,997,180-sat change output, leaving 2,820 sats as the fee. The receiver output was
identified by its known address, and the other standard output was change back to the
miner wallet. There is no separate fee output; the fee is the difference between the
input and output totals.
