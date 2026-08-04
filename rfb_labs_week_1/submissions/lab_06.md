# Lab 06 — Transaction decoding

## Commands used
- `bitcoin-cli -regtest getrawtransaction <txid> 2` backs `decode_verbose_transaction`
- `input_outpoints` (Rust) maps the decoded input to its outpoint
- `identify_payment_and_change` (Rust) matches the receiver address against outputs
- `calculate_fee` (Rust) computes `sum(inputs) - sum(outputs)`

## Terminal output

$ bitcoin-cli -regtest getrawtransaction c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54 2
{
"txid": "c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54",
"hash": "87a377abe51f1e8634eeb54348c08fa1afff49126124bc2bd15e8b3473528282",
"version": 2,
"size": 222,
"vsize": 141,
"weight": 561,
"locktime": 688,
"vin": [
{
"txid": "2a6039afca268e69258fd755d7f8254fa4d0dd8cf002f1d7049fe08dbf8ce1c8",
"vout": 0,
"scriptSig": { "asm": "", "hex": "" },
"txinwitness": ["304402206df933b8e86deb656fc406cd109bc811b1e620bd7e9eeda080a56075e894923d02202e14869ec3e126222e4267059502b285a2c1e3270296112293311422a86ad30601",
"0237e46bb8f87939a87634f2bb2a7cc626a7bbb653a0f1cf2d2cf43c6022c05c66"
],
"sequence": 4294967293
}
],
"vout": [
{
"value": 5.24998590,
"n": 0,
"scriptPubKey": {"address": "bcrt1qc7szugpdj82tkz8lw8dreced6jrewwwwxgfth2",
"hex": "0014c7a02e202d91d4bb08ff71da3ce32dd4879739ce",
"type": "witness_v0_keyhash"
}
},
{
"value": 1.00000000,
"n": 1,
"scriptPubKey": {
"address": "bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr",
"hex": "00143a97bdcf69f7804db69646adef9236af6b40a427",
"type": "witness_v0_keyhash"
}
}
]
}

Manual decomposition:
- Input (outpoint): `txid = 2a6039afca268e69258fd755d7f8254fa4d0dd8cf002f1d7049fe08dbf8ce1c8, vout = 0`,
  value = 6.25000000 BTC (this is the matured coinbase UTXO from Lab 03/04).
- Output `n=1`, value 1.00000000 BTC, address `bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr`
  — this matches `receiver`'s address from Lab 02, so this is the **payment**.
- Output `n=0`, value 5.24998590 BTC, address `bcrt1qc7szugpdj82tkz8lw8dreced6jrewwwwxgfth2`
  — a fresh address not previously used, generated automatically by the `miner`
  wallet to return leftover value — this is the **change**.
- Fee = total input − total output = 6.25000000 − (5.24998590 + 1.00000000)
  = **0.00001410 BTC**, which exactly matches the `fee: -0.00001410` field
  reported by `gettransaction` in Lab 05.

## Evidence references (co-authored by Claude)

Captured directly from the local regtest node, decoding the same transaction
broadcast in Lab 05 (`c5c0505f...`). The input value was cross-referenced
against the UTXO documented in Lab 04 (the matured 6.25 BTC coinbase).

## Explanation

Every Bitcoin transaction must obey a conservation rule: the total value of its outputs can never exceed the total value of its inputs. In this transaction, the single 6.25 BTC input splits into two outputs 1 BTC to the payment address and 5.24998590 BTC back to the sender as change and those two outputs sum to slightly less than the input. That small shortfall, 0.00001410 BTC, is the miner fee.

Unlike the payment or change amounts, the fee has no dedicated output anywhere in the transaction there's no line item for it. It exists only implicitly as the leftover: whatever value isn't explicitly assigned to an output is automatically claimed by whichever miner includes the transaction in a block. This is why calculating a fee requires looking up the input's original value (since the transaction itself only references which output it's spending, not its amount) and subtracting the sum of the stated outputs from it.

This means value conservation and fee calculation are the same arithmetic: sum(inputs) sum(outputs) = fee. Because Bitcoin's consensus rules reject any transaction that tries to create value from nothing, this difference can never be negative a transaction can only pay a fee, never demand one back. This also explains why the fee computed here (0.00001410) exactly matches what gettransaction independently reported in Lab 05 both are the same underlying accounting, just derived two different ways.