# Lab 04 — UTXOs and outpoints

## Commands used
- `bitcoin-cli -regtest -rpcwallet=miner listunspent` backs `list_unspent`
- `select_spendable_utxo` (Rust) filters by `spendable` and picks the most-confirmed entry
- `outpoint` (Rust) extracts the `txid:vout` coordinate from a UTXO
- `sum_spendable_utxos` (Rust) sums the `amount` field across spendable entries

## Terminal output
$ bitcoin-cli -regtest -rpcwallet=miner listunspent
[
{
"txid":
"2a6039afca268e69258fd755d7f8254fa4d0dd8cf002f1d7049fe08dbf8ce1c8",
"vout": 0,
"address": "bcrt1q7qcth6f0cq4w5tnfruqcxu20js6dcxrrmgum2z",
"label": "miner-address",
"scriptPubKey": "0014f030bbe92fc02aea2e691f0183714f9434dc1863",
"amount": 6.25000000,
"confirmations": 101,
"spendable": true,
"solvable": true,
"desc":
"wpkh([06312982/84'/1'/0'/0/0]0237e46bb8f87939a87634f2bb2a7cc626a7bbb653a0f1cf2d2cf43c6022c05c66)#0xqarsg2",
"parent_descs": [
"wpkh(tpubD6NzVbkrYhZ4Xcbkck2ALkwzouGSiiGaqNb44sN5njJCCu49fA9Y5LEUt2Ksw4vsv7VqvD5kWSqd5jPzj8caZ34dytXYHwZhdk4Wj1CrFsL/84'/1'/0'/0/*)#x4kgn4a9",
"safe": true
}
]
Derived values from this UTXO:
- Outpoint: `txid = 2a6039afca268e69258fd755d7f8254fa4d0dd8cf002f1d7049fe08dbf8ce1c8`, `vout = 0`
- Spendable: `true`, confirmations: `101` (this is the reward that matured in Lab 03)
- Sum of spendable UTXOs in this wallet: `6.25000000` BTC (only one entry present)

## Evidence references
Captured directly from the `miner` wallet on the same local regtest node used
in Lab 03. This is the single coinbase reward that matured after mining 100
additional blocks its 101 confirmations and `spendable: true` status confirm
it crossed the maturity threshold and can now be selected as a transaction input.

## Explanation (co-authored by Claude)

Bitcoin doesn't track balances the way a bank account does, where a single number goes up and down. Instead, every bitcoin in existence lives inside a UTXO, an Unspent Transaction Output which is essentially a discrete "coin" created as an output of some earlier transaction and not yet spent by any later one. A wallet's balance isn't stored anywhere as a single figure; it's calculated by summing up the amounts of every UTXO that wallet currently controls the private key for. In the evidence above, the miner wallet's entire balance is just one UTXO worth 6.25 BTC the exact coinbase reward that matured in Lab 03 - so sum_spendable_utxos over this single-entry list correctly returns 6.25.

Each UTXO is uniquely and permanently identified by its outpoint: the transaction ID that created it, paired with the output index (vout) within that transaction, since a single transaction can create multiple outputs. Here the outpoint is txid = 2a6039af...ce1c8, vout = 0 this pair is what a future transaction would reference as an input if it wanted to spend this exact coin. This is different from an address, which just identifies who can spend a UTXO the outpoint identifies which specific UTXO is being spent.

Not every UTXO a wallet knows about is necessarily usable right away. listunspent reports a spendable flag (and separately, confirmation depth) precisely because some UTXOs like an immature coinbase reward technically belong to the wallet but can't yet be used as a transaction input. That's why select_spendable_utxo filters on spendable before picking a coin to use, and why the 101-confirmation, spendable: true UTXO in this evidence is exactly what we'd expect to see for a reward that just crossed the 100-block maturity threshold.

