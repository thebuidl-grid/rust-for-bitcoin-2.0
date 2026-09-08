# Lab 04 — UTXOs and outpoints

## Commands used

```
cargo test --test lab_04

bitcoin-cli -regtest -rpcwallet=miner listunspent
```

*RPC is the one issued by `list_unspent` in `src/labs/lab04_utxos.rs`; `select_spendable_utxo`, `outpoint`, and `sum_spendable_utxos` operate on the decoded UTXOs locally (no further RPCs), verified against the mocked RPC client in `tests/lab_04.rs`. Run the `bitcoin-cli` line against your live Polar regtest node to capture the terminal output below.*

## Terminal output

Captured against the live regtest node at height 101 (continuing from Lab 03):

```
$ bitcoin-cli -regtest -rpcwallet=miner listunspent
[
  {
    "txid": "c313884361a36760928891af56c5dfb4cca60bf7bea888640528e90e5cbac1ec",
    "vout": 0,
    "address": "bcrt1qtdwur5a220ta9f0lndtdqf45f0kmaplt3xea5l",
    "label": "faucet",
    "scriptPubKey": "00145b5dc1d3aa53d7d2a5ff9b56d026b44bedbe87eb",
    "amount": 50.00000000,
    "confirmations": 101,
    "spendable": true,
    "solvable": true,
    "safe": true
  }
]
```

Only one UTXO shows up: the block-1 coinbase, now at 101 confirmations and mature. The other 100 coinbases mined in Lab 03 are excluded entirely — `listunspent` doesn't report immature coinbase outputs at all, which matches why `get_balances`' `trusted` field was exactly this one output's 50 BTC.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a bitcoin-cli ...` against a real regtest node (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

Bitcoin doesn't track account balances the way a bank does. Instead, every transaction *consumes* some previous outputs and *creates* new ones. An output that hasn't been consumed yet is called a **UTXO** — an Unspent Transaction Output. Owning bitcoin literally means holding the private key that can unlock one or more UTXOs.

An **outpoint** is simply the unique address of a UTXO: the pair `(txid, vout)` — which transaction created it, and which output position (`vout`, 0-indexed) within that transaction it is. Since a transaction can have several outputs, `txid` alone isn't enough to identify one; the outpoint's `vout` says exactly which one. That's the `txid`/`vout` pair every UTXO in `listunspent` carries.

A **wallet balance is the sum of every UTXO the wallet can spend**, because there's nothing else it could be — there's no running ledger entry to add up, only a set of discrete, spendable outputs. That's why `listunspent` returning a single 50 BTC UTXO (block 1's now-mature coinbase, outpoint `c313884...ac1ec:0`) lines up exactly with `get_balances`'s `trusted: 50.00000000` from Lab 03: one UTXO, one output, the whole balance. Spending later combines and splits UTXOs — Lab 09 shows a spend consuming three separate UTXOs at once — but the balance is always just "sum up what's still unspent and spendable."
