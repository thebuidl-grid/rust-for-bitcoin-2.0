# Lab 04 — UTXOs and outpoints

## Commands used

Rust: `cargo run --example run` (calls the following):
- `listunspent` (wallet: miner) — via `list_unspent`
- `getbalances` (wallet: miner) — via `get_balances`, to reconcile against the UTXO sum

## Terminal output

=== Lab 04: UTXOs ===
selected UTXO:
txid: 3d8cdd5803e49ad0eb24c9b7c1e1ae713e9cc37b40fd034fd3d05a31950cc820
vout: 0
amount: 50
confirmations: 101
address: Some("bcrt1qmy6upf6793ax80sjque9tx2hjwsxwe3phcf50f")
script_pub_key: 0014d935c0a5fe2c7a63be12073255995793a0676621
spendable: true
outpoint: OutPoint { txid: "3d8cdd5803e49ad0eb24c9b7c1e1ae713e9cc37b40fd034fd3d05a31950cc820", vout: 0 }
sum of spendable UTXOs: 50
wallet trusted balance: 50
reconciles with wallet balance: true

## Evidence references

Screenshot: `evidence/lab04.png`

## Explanation

A UTXO (Unspent Transaction Output) is the actual unit of value Bitcoin moves — not an account balance, but a discrete, indivisible chunk of coin sitting at a specific address, waiting to be spent. Every coin I own exists as one or more of these outputs.

An outpoint is what uniquely identifies a single UTXO across the entire blockchain: the `txid` of the transaction that created it, paired with the `vout` index of that specific output within that transaction (since one transaction can create several outputs at once). My selected UTXO's outpoint is `3d8cdd...cc820:0` — that exact pair is how a future transaction would reference this coin as an input if it wanted to spend it.

Bitcoin Core has no concept of a stored "account balance." What `getbalances` reports is derived on the fly by scanning every UTXO the wallet's keys can unlock and adding up their amounts. My single 50 BTC UTXO (mature, spendable, 101 confirmations) is the only spendable output the `miner` wallet controls at this point, so `sum of spendable UTXOs` (computed independently by summing every spendable UTXO's amount myself) landed on exactly 50 — matching the wallet's own reported `trusted` balance. That reconciliation is proof the wallet's balance figure isn't some separately-tracked number; it's genuinely just the sum of the UTXOs underneath it
